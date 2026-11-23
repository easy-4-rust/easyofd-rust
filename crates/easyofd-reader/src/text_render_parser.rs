//! 转换文字元数据解析，复用核心文字类型。
use crate::parser::{doc_path, read_zip_entry};
use crate::text_render_run::TextRenderRun;
use easyofd_core::{CT_CGTransform, CT_Text, OfdError, OfdResult, TextCode, XmlNode};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};

fn bad(message: impl Into<String>) -> OfdError {
    OfdError::Xml(message.into())
}
fn number(value: &str) -> OfdResult<f64> {
    let n: f64 = value
        .parse()
        .map_err(|_| bad(format!("无效数值: {value}")))?;
    if !n.is_finite() {
        return Err(bad("数值必须有限"));
    }
    Ok(n)
}
fn values(value: &str) -> OfdResult<Vec<f64>> {
    value.split_whitespace().map(number).collect()
}
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 已校验有限且0..255的颜色分量。
fn color(node: &XmlNode, key: &str) -> OfdResult<Option<u32>> {
    let Some(s) = node.child(key).and_then(|n| n.get_attr("Value")) else {
        return Ok(None);
    };
    let v = values(s)?;
    if v.len() != 3 || v.iter().any(|x| *x < 0. || *x > 255.) {
        return Err(bad("无效RGB颜色"));
    }
    Ok(Some(
        ((v[0] as u32) << 16) | ((v[1] as u32) << 8) | (v[2] as u32),
    ))
}
fn boolean(node: &XmlNode, key: &str, default: bool) -> OfdResult<bool> {
    match node.get_attr(key) {
        None => Ok(default),
        Some("true" | "1") => Ok(true),
        Some("false" | "0") => Ok(false),
        _ => Err(bad(format!("无效布尔值: {key}"))),
    }
}
fn delta(value: Option<&str>, count: usize) -> OfdResult<Vec<f64>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let tokens: Vec<_> = value.split_whitespace().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "g" {
            let n: usize = tokens
                .get(i + 1)
                .ok_or_else(|| bad("delta缺少次数"))?
                .parse()
                .map_err(|_| bad("delta次数无效"))?;
            let v = number(tokens.get(i + 2).ok_or_else(|| bad("delta缺少值"))?)?;
            if n > count.saturating_sub(out.len()) {
                return Err(bad("delta重复次数超过文字长度"));
            }
            out.extend(std::iter::repeat_n(v, n));
            i += 3;
        } else {
            out.push(number(tokens[i])?);
            i += 1;
        }
        if out.len() > count {
            return Err(bad("delta超过文字长度"));
        }
    }
    Ok(out)
}
fn descendants<'a>(node: &'a XmlNode, name: &str, out: &mut Vec<&'a XmlNode>) {
    if node.name == name {
        out.push(node);
    }
    for c in &node.children {
        descendants(c, name, out);
    }
}
fn inherited<'a>(id: &str, draws: &'a HashMap<String, XmlNode>) -> OfdResult<Vec<&'a XmlNode>> {
    let mut seen = HashSet::new();
    let mut chain = Vec::new();
    let mut next = Some(id);
    while let Some(key) = next {
        if !seen.insert(key) {
            return Err(bad("DrawParam继承循环"));
        }
        let n = draws
            .get(key)
            .ok_or_else(|| bad(format!("缺少DrawParam: {key}")))?;
        chain.push(n);
        next = n.get_attr("Relative");
    }
    chain.reverse();
    Ok(chain)
}
/// 从页面XML和实际资源引用提取绘制信息，保留原始模型路径。
pub(crate) fn parse_runs<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    doc_dir: &str,
    document: &str,
    page_path: &str,
    page: &XmlNode,
    media_ids: &HashSet<String>,
) -> OfdResult<Vec<TextRenderRun>> {
    let doc = easyofd_core::parse_xml_to_nodes(document).map_err(bad)?;
    let mut res_refs = Vec::new();
    descendants(&doc, "PublicRes", &mut res_refs);
    descendants(&doc, "DocumentRes", &mut res_refs);
    let mut resource_paths: Vec<String> = res_refs
        .iter()
        .filter_map(|n| n.text.as_ref())
        .map(|s| doc_path(doc_dir, s.trim()))
        .collect();
    let mut page_refs = Vec::new();
    descendants(page, "PageRes", &mut page_refs);
    let parent = page_path.rsplit_once('/').map_or("", |x| x.0);
    resource_paths.extend(
        page_refs
            .iter()
            .filter_map(|n| n.text.as_ref())
            .map(|s| doc_path(parent, s.trim())),
    );
    let mut draws = HashMap::new();
    let mut fonts = HashMap::new();
    for path in resource_paths {
        let bytes = read_zip_entry(archive, &path)?;
        let res = easyofd_core::parse_xml_to_nodes(
            std::str::from_utf8(&bytes).map_err(|_| bad("资源XML编码错误"))?,
        )
        .map_err(bad)?;
        let mut nodes = Vec::new();
        descendants(&res, "DrawParam", &mut nodes);
        for n in nodes {
            if let Some(id) = n.get_attr("ID") {
                draws.insert(id.to_owned(), n.clone());
            }
        }
        let mut nodes = Vec::new();
        descendants(&res, "Font", &mut nodes);
        let base = res.get_attr("BaseLoc").unwrap_or("");
        let dir = path.rsplit_once('/').map_or("", |x| x.0);
        for n in nodes {
            if let Some(id) = n.get_attr("ID") {
                let name = n.get_attr("FontName").map(str::to_owned);
                let data = if let Some(file) = n.child("FontFile").and_then(|f| f.text.as_ref()) {
                    Some(read_zip_entry(
                        archive,
                        &doc_path(&doc_path(dir, base), file.trim()),
                    )?)
                } else {
                    None
                };
                fonts.insert(id.to_owned(), (name, data));
            }
        }
    }
    let mut out = Vec::new();
    let mut index = 0;
    walk(page, &draws, &fonts, media_ids, &mut index, &mut out)?;
    Ok(out)
}
fn walk(
    node: &XmlNode,
    draws: &HashMap<String, XmlNode>,
    fonts: &HashMap<String, (Option<String>, Option<Vec<u8>>)>,
    media_ids: &HashSet<String>,
    index: &mut usize,
    out: &mut Vec<TextRenderRun>,
) -> OfdResult<()> {
    if node.name == "TextObject" {
        let boundary = node.get_attr("Boundary").unwrap_or("0 0 0 0");
        if values(boundary)?.len() != 4 {
            return Err(bad("文字Boundary需四项"));
        }
        let mut t = CT_Text::new(
            node.get_attr("ID")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            boundary,
        );
        t.stroke = boolean(node, "Stroke", false)?;
        t.fill = boolean(node, "Fill", true)?;
        t.size = node.get_attr("Size").map(number).transpose()?;
        t.font_ref = node.get_attr("Font").and_then(|s| s.parse().ok());
        t.h_scale = node.get_attr("HScale").map(number).transpose()?;
        t.read_direction = node.get_attr("ReadDirection").and_then(|s| s.parse().ok());
        t.char_direction = node.get_attr("CharDirection").and_then(|s| s.parse().ok());
        let mut width = 0.353;
        let mut chain = if let Some(id) = node.get_attr("DrawParam") {
            inherited(id, draws)?
        } else {
            Vec::new()
        };
        chain.push(node);
        for n in chain {
            if let Some(c) = color(n, "StrokeColor")? {
                t.stroke_color = Some(c);
            }
            if let Some(c) = color(n, "FillColor")? {
                t.fill_color = Some(c);
            }
            if let Some(s) = n.get_attr("LineWidth") {
                width = number(s)?;
                if width < 0. {
                    return Err(bad("负线宽"));
                }
            }
        }
        for n in &node.children {
            if n.name == "TextCode" {
                let mut code = TextCode::with_content(n.text.as_deref().unwrap_or(""));
                code.x = n.get_attr("X").map(number).transpose()?;
                code.y = n.get_attr("Y").map(number).transpose()?;
                let count = code.content.chars().count();
                code.delta_x = delta(n.get_attr("DeltaX"), count)?;
                code.delta_y = delta(n.get_attr("DeltaY"), count)?;
                t.text_codes.push(code);
            } else if n.name == "CGTransform" {
                let mut cg = CT_CGTransform::new();
                cg.code_position = n.get_attr("CodePosition").and_then(|s| s.parse().ok());
                cg.code_count = n.get_attr("CodeCount").and_then(|s| s.parse().ok());
                cg.glyph_count = n.get_attr("GlyphCount").and_then(|s| s.parse().ok());
                if let Some(s) = n
                    .get_attr("Glyphs")
                    .or_else(|| n.child("Glyphs").and_then(|c| c.text.as_deref()))
                {
                    cg.glyphs = s
                        .split_whitespace()
                        .map(|v| v.parse().map_err(|_| bad("无效Glyphs")))
                        .collect::<OfdResult<_>>()?;
                }
                t.cg_transforms.push(cg);
            }
        }
        let ctm = node
            .get_attr("CTM")
            .map(|s| {
                let v = values(s)?;
                v.try_into().map_err(|_| bad("CTM需六项"))
            })
            .transpose()?;
        let (font_name, font_data) = node
            .get_attr("Font")
            .and_then(|id| fonts.get(id))
            .cloned()
            .unwrap_or_default();
        out.push(TextRenderRun {
            content_index: *index,
            text: t,
            ctm,
            line_width_mm: width,
            font_name,
            font_data,
        });
        *index += 1;
    } else if node.name == "PathObject" {
        *index += 1;
    } else if node.name == "ImageObject" {
        if node
            .get_attr("ResourceID")
            .is_some_and(|id| media_ids.contains(id))
        {
            *index += 1;
        }
    } else {
        for c in &node.children {
            walk(c, draws, fonts, media_ids, index, out)?;
        }
    }
    Ok(())
}
