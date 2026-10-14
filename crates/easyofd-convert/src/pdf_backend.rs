//! 将既有逐字绘制契约映射为 printpdf 操作流，保留字体子集及 Unicode 映射。
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::rc::Rc;

pub(crate) use printpdf::{
    BuiltinFont, Color, Line, LinePoint, Mm, Point, Rgb, TextMatrix, TextRenderingMode,
};
use printpdf::{
    Codepoint, FontId, Op, PaintMode, ParsedFont, PdfFontHandle, PdfPage, PdfSaveOptions, Polygon,
    PolygonRing, Pt, RawImage, RawImageData, RawImageFormat, TextItem, WindingOrder,
    XObjectTransform,
};

/// 文档与操作流共享句柄；仅在同步转换内部使用。
pub(crate) struct PdfDocumentReference(
    Rc<RefCell<printpdf::PdfDocument>>,
    Rc<RefCell<HashMap<FontId, Vec<GlyphFontVariant>>>>,
);
/// 同一字体的显式字形映射变体，隔离同一 gid 的不同 Unicode。
struct GlyphFontVariant {
    font: FontId,
    mappings: HashMap<u16, char>,
}
/// 页面绘制句柄，将绘制调用顺序保存在所属页面。
#[derive(Clone)]
pub(crate) struct PdfLayerReference {
    document: Rc<RefCell<printpdf::PdfDocument>>,
    page: usize,
    glyph_fonts: Rc<RefCell<HashMap<FontId, Vec<GlyphFontVariant>>>>,
}
/// 外部或内置字体的明确身份。
pub(crate) type IndirectFontRef = PdfFontHandle;
/// 文档构建入口，保留现有内部调用形式。
pub(crate) struct PdfDocument;
impl PdfDocument {
    /// 创建含首个页面的文档，返回文档、页号及内部层标记。
    pub(crate) fn create(
        title: &str,
        width: Mm,
        height: Mm,
        _layer: &str,
    ) -> (PdfDocumentReference, usize, usize) {
        let mut doc = printpdf::PdfDocument::new(title);
        doc.pages.push(PdfPage::new(width, height, Vec::new()));
        (
            PdfDocumentReference(
                Rc::new(RefCell::new(doc)),
                Rc::new(RefCell::new(HashMap::new())),
            ),
            0,
            0,
        )
    }
}
impl PdfDocumentReference {
    /// 返回页面操作句柄；参数是内部页号。
    pub(crate) fn get_page(&self, page: usize) -> PdfLayerReference {
        PdfLayerReference {
            document: Rc::clone(&self.0),
            page,
            glyph_fonts: Rc::clone(&self.1),
        }
    }
    /// 追加指定毫米尺寸的页面，返回内部页号及层标记。
    pub(crate) fn add_page(&self, width: Mm, height: Mm, _layer: &str) -> (usize, usize) {
        let mut doc = self.0.borrow_mut();
        let index = doc.pages.len();
        doc.pages.push(PdfPage::new(width, height, Vec::new()));
        (index, 0)
    }
    /// 注册内置字体；返回可直接供 Tf 使用的句柄。
    pub(crate) fn add_builtin_font(font: BuiltinFont) -> Result<IndirectFontRef, String> {
        Ok(PdfFontHandle::Builtin(font))
    }
    /// 解析字体并注册，参数读取器保留全部原字体字节供子集化使用。
    pub(crate) fn add_external_font(
        &self,
        mut reader: impl Read,
    ) -> Result<IndirectFontRef, String> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        let parsed = ParsedFont::from_bytes(&bytes, 0, &mut Vec::new())
            .ok_or_else(|| "字体解析失败".to_string())?;
        Ok(PdfFontHandle::External(
            self.0.borrow_mut().add_font(&parsed),
        ))
    }
    /// 加载外部字体；所有输出统一启用子集，参数用于兼容既有调用。
    pub(crate) fn add_external_font_with_subsetting(
        &self,
        reader: impl Read,
        _subset: bool,
    ) -> Result<IndirectFontRef, String> {
        self.add_external_font(reader)
    }
    /// 将操作流序列化并写入输出，保留子集化及真实位图。
    pub(crate) fn save(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let mut document = self.0.borrow_mut();
        defer_font_operations(&mut document);
        let bytes = document.save(
            &PdfSaveOptions {
                optimize: false,
                subset_fonts: true,
                image_optimization: None,
                ..Default::default()
            },
            &mut Vec::new(),
        );
        write_canonical_text_pdf(&bytes, writer)?;
        writer.flush()
    }
}
/// 单字符串 TJ 与 Tj 完全等价；规范化可避免通用提取器把数组边界误判为空格。
fn write_canonical_text_pdf(bytes: &[u8], writer: &mut impl Write) -> std::io::Result<()> {
    if !bytes.windows(4).any(|window| window == b"] TJ") {
        return writer.write_all(bytes);
    }
    let mut pdf = lopdf::Document::load_mem(bytes).map_err(std::io::Error::other)?;
    let mut changed = false;
    for id in pdf.get_pages().values() {
        let mut content = lopdf::content::Content::decode(&pdf.get_page_content(*id))
            .map_err(std::io::Error::other)?;
        let mut page_changed = false;
        for operation in &mut content.operations {
            if operation.operator == "TJ" && operation.operands.len() == 1 {
                if let Ok(items) = operation.operands[0].as_array() {
                    if items.len() == 1 && items[0].as_str().is_ok() {
                        // 只改等价运算符，不解码/改写字节；glyph CID 和实际空格原样保留。
                        operation.operator = "Tj".to_string();
                        operation.operands = vec![items[0].clone()];
                        page_changed = true;
                    }
                }
            }
        }
        if page_changed {
            pdf.change_page_content(*id, content.encode().map_err(std::io::Error::other)?)
                .map_err(std::io::Error::other)?;
            changed = true;
        }
    }
    if changed {
        pdf.save_to(writer)
    } else {
        writer.write_all(bytes)
    }
}

/// 仅在真实文本输出前设置字体，避免未使用的字体被底层省略后留下悬空 Tf。
fn defer_font_operations(document: &mut printpdf::PdfDocument) {
    for page in &mut document.pages {
        let mut current = None;
        let mut emitted = None;
        let mut stack = Vec::new();
        for operation in std::mem::take(&mut page.ops) {
            match operation {
                Op::SetFont { font, size } => current = Some((font, size)),
                Op::ShowText { items } => {
                    if current != emitted {
                        if let Some((font, size)) = &current {
                            page.ops.push(Op::SetFont {
                                font: font.clone(),
                                size: *size,
                            });
                        }
                        emitted.clone_from(&current);
                    }
                    page.ops.push(Op::ShowText { items });
                }
                Op::SaveGraphicsState => {
                    stack.push(current.clone());
                    page.ops.push(Op::SaveGraphicsState);
                }
                Op::RestoreGraphicsState => {
                    if let Some(saved_current) = stack.pop() {
                        current = saved_current;
                    }
                    // PDF 阅读器恢复 q/Q 的字体，但 printpdf 的编码器不恢复；下一次输出必须同步 Tf。
                    emitted = None;
                    page.ops.push(Op::RestoreGraphicsState);
                }
                other => page.ops.push(other),
            }
        }
    }
}

impl PdfLayerReference {
    fn push(&self, operation: Op) {
        self.document.borrow_mut().pages[self.page]
            .ops
            .push(operation);
    }
    /// 返回当前页面绘制句柄；内部只使用单层。
    pub(crate) fn get_layer(self, _layer: usize) -> Self {
        self
    }
    /// 保存当前绘图状态。
    pub(crate) fn save_graphics_state(&self) {
        self.push(Op::SaveGraphicsState);
    }
    /// 恢复绘图状态。
    pub(crate) fn restore_graphics_state(&self) {
        self.push(Op::RestoreGraphicsState);
    }
    /// 设置填充颜色。
    pub(crate) fn set_fill_color(&self, col: Color) {
        self.push(Op::SetFillColor { col });
    }
    /// 设置描边颜色。
    pub(crate) fn set_outline_color(&self, col: Color) {
        self.push(Op::SetOutlineColor { col });
    }
    /// 设置以点为单位的描边宽度。
    pub(crate) fn set_outline_thickness(&self, pt: f32) {
        self.push(Op::SetOutlineThickness { pt: Pt(pt) });
    }
    /// 开始文本操作段。
    pub(crate) fn begin_text_section(&self) {
        self.push(Op::StartTextSection);
    }
    /// 结束文本操作段。
    pub(crate) fn end_text_section(&self) {
        self.push(Op::EndTextSection);
    }
    /// 设置字体与点字号。
    pub(crate) fn set_font(&self, font: &IndirectFontRef, size: f32) {
        self.push(Op::SetFont {
            font: font.clone(),
            size: Pt(size),
        });
    }
    /// 设置文字填充或描边模式。
    pub(crate) fn set_text_rendering_mode(&self, mode: TextRenderingMode) {
        self.push(Op::SetTextRenderingMode { mode });
    }
    /// 设置精确文本矩阵，保持 PDF 坐标与 CTM。
    pub(crate) fn set_text_matrix(&self, matrix: TextMatrix) {
        self.push(Op::SetTextMatrix { matrix });
    }
    /// 输出 Unicode 文本；字体已由 Tf 操作设置。
    pub(crate) fn write_text(&self, text: String, _font: &IndirectFontRef) {
        self.push(Op::ShowText {
            items: vec![TextItem::Text(text)],
        });
    }
    /// 输出显式字形并记录对应 Unicode，确保字形纳入字体子集。
    pub(crate) fn write_glyph(
        &self,
        gid: u16,
        character: char,
        source: &IndirectFontRef,
        size: f32,
    ) {
        let font = if let PdfFontHandle::External(source_id) = source {
            let mut pool = self.glyph_fonts.borrow_mut();
            let variants = pool.entry(source_id.clone()).or_default();
            // 专用变体不与普通文本共享 ToUnicode；兼容映射复用，冲突才追加字体资源。
            let variant = if let Some(index) = variants
                .iter()
                .position(|v| v.mappings.get(&gid).is_none_or(|c| *c == character))
            {
                &mut variants[index]
            } else {
                let mut doc = self.document.borrow_mut();
                let parsed = doc.resources.fonts.map[source_id].parsed_font.clone();
                let id = doc.add_font(&parsed);
                variants.push(GlyphFontVariant {
                    font: id,
                    mappings: HashMap::new(),
                });
                variants.last_mut().expect("刚追加的字体变体")
            };
            variant.mappings.insert(gid, character);
            PdfFontHandle::External(variant.font.clone())
        } else {
            source.clone()
        };
        self.set_font(&font, size);
        self.push(Op::ShowText {
            items: vec![TextItem::GlyphIds(vec![Codepoint::with_cid(
                gid,
                0.,
                character.to_string(),
            )])],
        });
        // 恢复普通文本字体及字号，避免显式映射污染后续文本或页面。
        self.set_font(source, size);
    }
    /// 在毫米坐标处输出指定字号的文本。
    pub(crate) fn use_text(&self, text: &str, size: f32, x: Mm, y: Mm, font: &IndirectFontRef) {
        self.begin_text_section();
        self.set_font(font, size);
        self.push(Op::SetTextCursor {
            pos: Point::new(x, y),
        });
        self.write_text(text.to_string(), font);
        self.end_text_section();
    }
    /// 绘制折线或闭合轮廓。
    pub(crate) fn add_paths(&self, paths: Vec<Line>, fill: bool) {
        if fill {
            // 所有轮廓共同执行非零填充，保留反向内环形成的孔洞。
            self.push(Op::DrawPolygon {
                polygon: Polygon {
                    rings: paths
                        .iter()
                        .map(|line| PolygonRing {
                            points: line.points.clone(),
                        })
                        .collect(),
                    mode: PaintMode::Fill,
                    winding_order: WindingOrder::NonZero,
                },
            });
        }
        // 填充隐式闭合与描边开闭分别处理；开放路径不出现新增的闭合描边。
        for line in paths {
            self.push(Op::DrawLine { line });
        }
    }
    /// 嵌入真实 RGBA 像素；尺寸及底部坐标从 OFD 毫米值换算。
    pub(crate) fn add_rgba_image(
        &self,
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        image: &easyofd_core::ImageObject,
        page_height: f64,
    ) {
        let raw = RawImage {
            pixels: RawImageData::U8(pixels),
            width: width as usize,
            height: height as usize,
            data_format: RawImageFormat::RGBA8,
            tag: Vec::new(),
        };
        let id = self.document.borrow_mut().add_image(&raw);
        self.push(Op::UseXobject {
            id,
            transform: XObjectTransform {
                translate_x: Some(Mm(image.x as f32).into()),
                translate_y: Some(Mm((page_height - image.y - image.height) as f32).into()),
                scale_x: Some(image.width as f32 * 300. / (width as f32 * 25.4)),
                scale_y: Some(image.height as f32 * 300. / (height as f32 * 25.4)),
                dpi: Some(300.),
                ..Default::default()
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{BuiltinFont, Color, Mm, PdfDocument, Rgb};
    use std::io::{Cursor, Read};

    fn fixture_font() -> Vec<u8> {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/upstream_20261014/stroke_true.ofd");
        let mut zip = zip::ZipArchive::new(Cursor::new(std::fs::read(file).unwrap())).unwrap();
        let mut font = Vec::new();
        zip.by_name("Doc_0/Res/font.ttf")
            .unwrap()
            .read_to_end(&mut font)
            .unwrap();
        font
    }

    #[test]
    fn explicit_glyph_is_subset_and_unicode_selectable() {
        let data = fixture_font();
        let face = crate::font::font_face::FontFace::parse(&data, 0).unwrap();
        let gid = face.glyph_index('/').unwrap().0;
        let (doc, page, layer) = PdfDocument::create("Glyph Unicode", Mm(100.), Mm(100.), "Layer");
        let original_glyphs = face.number_of_glyphs();
        let original_size = data.len();
        let font = doc.add_external_font(Cursor::new(data)).unwrap();
        let layer = doc.get_page(page).get_layer(layer);
        layer.begin_text_section();
        layer.set_font(&font, 12.);
        layer.write_glyph(gid, '中', &font, 12.);
        layer.end_text_section();
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(pdf.extract_text(&[1]).unwrap().trim(), "中");
        let embedded = pdf
            .objects
            .values()
            .filter_map(|object| object.as_stream().ok())
            .find(|stream| stream.dict.get(b"Length1").is_ok())
            .unwrap();
        let font_bytes = if embedded.dict.get(b"Filter").is_ok() {
            embedded.decompressed_content().unwrap()
        } else {
            embedded.content.clone()
        };
        let subset = crate::font::font_face::FontFace::parse(&font_bytes, 0).unwrap();
        assert!(
            subset.number_of_glyphs() < original_glyphs / 10,
            "must embed a true subset, not the full font"
        );
        assert!(
            font_bytes.len() < original_size / 10,
            "subset bytes must measurably shrink"
        );
        assert!(
            subset.number_of_glyphs() >= 2,
            "used glyph plus .notdef must survive"
        );
    }

    /// 基于已有真实字体构造 cmap 不可达的复合字形；复合组件指向原 slash。
    fn composite_unmapped_font() -> (Vec<u8>, u16) {
        let original = fixture_font();
        let face = crate::font::font_face::FontFace::parse(&original, 0).unwrap();
        let child = face.glyph_index('/').unwrap().0;
        let parent = face.glyph_index('A').unwrap().0;
        let raw = &face.font;
        let loca = raw
            .table_data(skrifa::raw::types::Tag::new(b"loca"))
            .unwrap()
            .as_bytes();
        let head = raw
            .table_data(skrifa::raw::types::Tag::new(b"head"))
            .unwrap()
            .as_bytes();
        let offset = |gid: u16| {
            if i16::from_be_bytes([head[50], head[51]]) == 0 {
                let i = usize::from(gid) * 2;
                usize::from(u16::from_be_bytes([loca[i], loca[i + 1]])) * 2
            } else {
                let i = usize::from(gid) * 4;
                u32::from_be_bytes(loca[i..i + 4].try_into().unwrap()) as usize
            }
        };
        let mut glyf = raw
            .table_data(skrifa::raw::types::Tag::new(b"glyf"))
            .unwrap()
            .as_bytes()
            .to_vec();
        let parent_start = offset(parent);
        let child_start = offset(child);
        assert!(offset(parent + 1) - parent_start >= 16);
        let bounds = glyf[child_start + 2..child_start + 10].to_vec();
        glyf[parent_start..offset(parent + 1)].fill(0);
        glyf[parent_start..parent_start + 2].copy_from_slice(&(-1_i16).to_be_bytes());
        glyf[parent_start + 2..parent_start + 10].copy_from_slice(&bounds);
        glyf[parent_start + 10..parent_start + 12].copy_from_slice(&2_u16.to_be_bytes());
        glyf[parent_start + 12..parent_start + 14].copy_from_slice(&child.to_be_bytes());
        // format 4 cmap 仅保留 slash，原 parent 不再通过任何 Unicode 可达。
        let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 1, 0, 0, 0, 12];
        for value in [
            4_u16,
            32,
            0,
            4,
            4,
            1,
            0,
            47,
            65535,
            0,
            47,
            65535,
            child.wrapping_sub(47),
            1,
            0,
            0,
        ] {
            cmap.extend_from_slice(&value.to_be_bytes());
        }
        let table_count = u16::from_be_bytes([original[4], original[5]]) as usize;
        let mut font = original[..12 + 16 * table_count].to_vec();
        let mut head_start = 0;
        let checksum = |bytes: &[u8]| {
            bytes.chunks(4).fold(0_u32, |sum, chunk| {
                let mut word = [0; 4];
                word[..chunk.len()].copy_from_slice(chunk);
                sum.wrapping_add(u32::from_be_bytes(word))
            })
        };
        for index in 0..table_count {
            let directory = 12 + index * 16;
            let tag: [u8; 4] = original[directory..directory + 4].try_into().unwrap();
            let mut bytes = match &tag {
                b"cmap" => cmap.clone(),
                b"glyf" => glyf.clone(),
                _ => {
                    let start = u32::from_be_bytes(
                        original[directory + 8..directory + 12].try_into().unwrap(),
                    ) as usize;
                    let len = u32::from_be_bytes(
                        original[directory + 12..directory + 16].try_into().unwrap(),
                    ) as usize;
                    original[start..start + len].to_vec()
                }
            };
            if &tag == b"head" {
                bytes[8..12].fill(0);
            }
            while !font.len().is_multiple_of(4) {
                font.push(0);
            }
            let start = font.len();
            if &tag == b"head" {
                head_start = start;
            }
            font[directory + 4..directory + 8].copy_from_slice(&checksum(&bytes).to_be_bytes());
            font[directory + 8..directory + 12].copy_from_slice(&(start as u32).to_be_bytes());
            font[directory + 12..directory + 16]
                .copy_from_slice(&(bytes.len() as u32).to_be_bytes());
            font.extend_from_slice(&bytes);
        }
        let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&font));
        font[head_start + 8..head_start + 12].copy_from_slice(&adjustment.to_be_bytes());
        (font, parent)
    }

    /// 记录实际解出的轮廓指令，确保复合子字形没有被子集化丢弃。
    #[derive(Default)]
    struct OutlineCommands(Vec<(u8, Vec<f32>)>);
    impl skrifa::outline::OutlinePen for OutlineCommands {
        fn move_to(&mut self, x: f32, y: f32) {
            self.0.push((0, vec![x, y]));
        }
        fn line_to(&mut self, x: f32, y: f32) {
            self.0.push((1, vec![x, y]));
        }
        fn quad_to(&mut self, x: f32, y: f32, x2: f32, y2: f32) {
            self.0.push((2, vec![x, y, x2, y2]));
        }
        fn curve_to(&mut self, x: f32, y: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
            self.0.push((3, vec![x, y, x2, y2, x3, y3]));
        }
        fn close(&mut self) {
            self.0.push((4, vec![]));
        }
    }

    fn draw_outline(
        face: &crate::font::font_face::FontFace<'_>,
        gid: u16,
        pen: &mut OutlineCommands,
    ) -> Result<(), skrifa::outline::DrawError> {
        use skrifa::MetadataProvider;
        face.font
            .outline_glyphs()
            .get(skrifa::GlyphId::new(u32::from(gid)))
            .unwrap()
            .draw(
                skrifa::outline::DrawSettings::unhinted(
                    skrifa::instance::Size::unscaled(),
                    skrifa::instance::LocationRef::default(),
                ),
                pen,
            )
            .map(|_| ())
    }

    #[test]
    fn subset_preserves_unmapped_composite_and_child_outline() {
        let (data, gid) = composite_unmapped_font();
        let face = crate::font::font_face::FontFace::parse(&data, 0).unwrap();
        assert!(face.glyph_index('A').is_none());
        let mut source = OutlineCommands::default();
        assert!(draw_outline(&face, gid, &mut source).is_ok());
        assert!(!source.0.is_empty());
        let (doc, page, layer) = PdfDocument::create("composite", Mm(100.), Mm(100.), "Layer");
        let font = doc.add_external_font(Cursor::new(data.clone())).unwrap();
        let layer = doc.get_page(page).get_layer(layer);
        layer.begin_text_section();
        layer.set_font(&font, 12.);
        layer.write_glyph(gid, '中', &font, 12.);
        layer.end_text_section();
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(
            pdf.extract_text_chunks(&[1])
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .concat(),
            "中\n"
        );
        let stream = pdf
            .objects
            .values()
            .filter_map(|object| object.as_stream().ok())
            .find(|stream| stream.dict.get(b"Length1").is_ok())
            .unwrap();
        let subset = if stream.dict.get(b"Filter").is_ok() {
            stream.decompressed_content().unwrap()
        } else {
            stream.content.clone()
        };
        let face = crate::font::font_face::FontFace::parse(&subset, 0).unwrap();
        assert_eq!(
            face.number_of_glyphs(),
            3,
            ".notdef, parent and its component must survive"
        );
        assert!(subset.len() < data.len() / 10);
        let content =
            lopdf::content::Content::decode(&pdf.get_page_content(pdf.get_pages()[&1])).unwrap();
        let operation = content
            .operations
            .iter()
            .find(|op| op.operator == "Tj")
            .unwrap();
        let encoded = operation.operands[0].as_str().unwrap();
        let subset_gid = u16::from_be_bytes(encoded.try_into().unwrap());
        let mut output = OutlineCommands::default();
        assert!(draw_outline(&face, subset_gid, &mut output).is_ok());
        assert_eq!(
            output.0, source.0,
            "subset preserves the complete composite outline"
        );
    }

    #[test]
    fn glyph_unicode_variants_survive_plain_text_and_page_boundaries() {
        let data = fixture_font();
        let gid = crate::font::font_face::FontFace::parse(&data, 0)
            .unwrap()
            .glyph_index('/')
            .unwrap()
            .0;
        let (doc, page, layer) =
            PdfDocument::create("mapping collision", Mm(100.), Mm(100.), "Layer");
        let font = doc.add_external_font(Cursor::new(data)).unwrap();
        let first = doc.get_page(page).get_layer(layer);
        first.begin_text_section();
        first.set_font(&font, 12.);
        first.write_text("/ ".to_string(), &font);
        first.write_glyph(gid, '中', &font, 12.);
        first.end_text_section();
        let (page, layer) = doc.add_page(Mm(100.), Mm(100.), "Layer");
        let second = doc.get_page(page).get_layer(layer);
        second.begin_text_section();
        second.set_font(&font, 12.);
        second.write_glyph(gid, '文', &font, 12.);
        second.write_text(" / ".to_string(), &font);
        second.write_glyph(gid, '中', &font, 12.);
        second.end_text_section();
        assert_eq!(
            doc.0.borrow().resources.fonts.map.len(),
            3,
            "two conflicting glyph mappings share two variants; repeated mapping reuses its font"
        );
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(
            pdf.extract_text_chunks(&[1])
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .concat(),
            "/ 中\n"
        );
        assert_eq!(
            pdf.extract_text_chunks(&[2])
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .concat(),
            "文 / 中\n"
        );
        for id in pdf.get_pages().values() {
            let fonts = pdf.get_page_fonts(*id).unwrap();
            let content = lopdf::content::Content::decode(&pdf.get_page_content(*id)).unwrap();
            for operation in content.operations.iter().filter(|op| op.operator == "Tf") {
                assert!((operation.operands[1].as_float().unwrap() - 12.).abs() < f32::EPSILON);
                assert!(
                    fonts.contains_key(operation.operands[0].as_name().unwrap()),
                    "every Tf must resolve to a font resource"
                );
            }
        }
    }

    /// 按 PDF q/Q 真正恢复字体状态，验证输出编码与阅读器状态一致。
    fn assert_font_after_restore(external: bool) {
        let data = fixture_font();
        let (doc, page, layer) = PdfDocument::create("font restore", Mm(100.), Mm(100.), "Layer");
        let first = if external {
            doc.add_external_font(Cursor::new(data.clone())).unwrap()
        } else {
            super::PdfDocumentReference::add_builtin_font(BuiltinFont::Helvetica).unwrap()
        };
        let embedded = doc.add_external_font(Cursor::new(data)).unwrap();
        let layer = doc.get_page(page).get_layer(layer);
        layer.use_text("before", 12., Mm(10.), Mm(80.), &first);
        layer.save_graphics_state();
        layer.use_text("embedded", 12., Mm(10.), Mm(60.), &embedded);
        layer.restore_graphics_state();
        layer.use_text("abc", 12., Mm(10.), Mm(40.), &first);
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        let id = pdf.get_pages()[&1];
        let fonts = pdf.get_page_fonts(id).unwrap();
        let content = lopdf::content::Content::decode(&pdf.get_page_content(id)).unwrap();
        let mut active = None;
        let mut stack = Vec::new();
        let mut text = String::new();
        for operation in &content.operations {
            match operation.operator.as_str() {
                "Tf" => {
                    let name = operation.operands[0].as_name().unwrap().to_vec();
                    assert!(fonts.contains_key(&name));
                    active = Some(name);
                }
                "q" => stack.push(active.clone()),
                "Q" => active = stack.pop().unwrap(),
                "Tj" => {
                    let encoding = fonts[active.as_ref().unwrap()]
                        .get_font_encoding(&pdf)
                        .unwrap();
                    encoding
                        .write_to_string(operation.operands[0].as_str().unwrap(), &mut text)
                        .unwrap();
                }
                "ET" => text.push('\n'),
                _ => {}
            }
        }
        assert_eq!(
            text, "before\nembedded\nabc\n",
            "PDF viewer font restoration must agree with the glyph encoder"
        );
        let restored = content
            .operations
            .iter()
            .position(|operation| operation.operator == "Q")
            .unwrap();
        let next_text = content.operations[restored + 1..]
            .iter()
            .position(|operation| operation.operator == "Tj")
            .unwrap()
            + restored
            + 1;
        let font = content.operations[restored + 1..next_text]
            .iter()
            .find(|operation| operation.operator == "Tf")
            .expect("explicit Tf after Q must synchronize the serializer");
        assert_eq!(
            font.operands[0].as_name().unwrap(),
            first.get_resource_name().as_bytes()
        );
        if let Some(dir) = std::env::var_os("EASYOFD_PDF_REVIEW_DIR") {
            let name = if external {
                "q-restore-external.pdf"
            } else {
                "q-restore-builtin.pdf"
            };
            std::fs::write(std::path::Path::new(&dir).join(name), bytes).unwrap();
        }
    }

    #[test]
    fn builtin_font_is_reselected_after_graphics_restore() {
        assert_font_after_restore(false);
    }
    #[test]
    fn external_font_is_reselected_after_graphics_restore() {
        assert_font_after_restore(true);
    }

    #[test]
    fn independent_pages_keep_their_dimensions_and_content() {
        let (doc, page, layer) = PdfDocument::create("Pages", Mm(100.), Mm(150.), "Layer");
        let font = super::PdfDocumentReference::add_builtin_font(BuiltinFont::Helvetica).unwrap();
        doc.get_page(page)
            .get_layer(layer)
            .use_text("first", 12., Mm(10.), Mm(20.), &font);
        let (page, layer) = doc.add_page(Mm(200.), Mm(250.), "Layer");
        doc.get_page(page)
            .get_layer(layer)
            .use_text("second", 12., Mm(10.), Mm(20.), &font);
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(pdf.get_pages().len(), 2);
        assert_eq!(pdf.extract_text(&[1]).unwrap().trim(), "first");
        assert_eq!(pdf.extract_text(&[2]).unwrap().trim(), "second");
        for (number, width) in [(1, 100.), (2, 200.)] {
            let id = pdf.get_pages()[&number];
            let bbox = pdf
                .get_object(id)
                .unwrap()
                .as_dict()
                .unwrap()
                .get(b"MediaBox")
                .unwrap()
                .as_array()
                .unwrap();
            assert!((f64::from(bbox[2].as_float().unwrap()) - width * 72. / 25.4).abs() < 0.01);
        }
    }

    #[test]
    fn filled_path_outputs_fill_and_stroke_operator() {
        let (doc, page, layer) = PdfDocument::create("Filled Path", Mm(100.), Mm(100.), "Layer");
        let layer = doc.get_page(page).get_layer(layer);
        layer.set_fill_color(Color::Rgb(Rgb::new(1., 0., 0., None)));
        let path = easyofd_core::PathObject::new(0., 0., "M 10 10 L 20 10 L 20 20 Z")
            .fill_color(0x00ff_0000);
        crate::render_path_to_pdf(&layer, &path, 100.);
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        let id = pdf.get_pages()[&1];
        let content = lopdf::content::Content::decode(&pdf.get_page_content(id)).unwrap();
        assert!(
            content
                .operations
                .iter()
                .any(|op| matches!(op.operator.as_str(), "f" | "f*")),
            "filled path must actually paint its fill"
        );
        let width = content
            .operations
            .iter()
            .find(|op| op.operator == "w")
            .unwrap()
            .operands[0]
            .as_float()
            .unwrap();
        assert!((f64::from(width) - path.stroke_width * 72. / 25.4).abs() < 0.001);
    }
}

#[cfg(test)]
mod normalization_regressions {
    use super::{Mm, PdfDocument, write_canonical_text_pdf};
    use printpdf::{Codepoint, Op, PdfSaveOptions, TextItem};
    use std::io::{Cursor, Read};

    #[test]
    fn normalization_preserves_resources_graphics_and_multi_item_tj() {
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/upstream_20261014/stroke_true.ofd");
        let mut zip = zip::ZipArchive::new(Cursor::new(std::fs::read(source).unwrap())).unwrap();
        let mut data = Vec::new();
        zip.by_name("Doc_0/Res/font.ttf")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        let gid = crate::font::font_face::FontFace::parse(&data, 0)
            .unwrap()
            .glyph_index('/')
            .unwrap()
            .0;
        let (doc, page, layer) = PdfDocument::create("preserve", Mm(100.), Mm(100.), "Layer");
        let font = doc.add_external_font(Cursor::new(data)).unwrap();
        let layer = doc.get_page(page).get_layer(layer);
        layer.save_graphics_state();
        layer.set_fill_color(printpdf::Color::Rgb(printpdf::Rgb::new(1., 0., 0., None)));
        layer.begin_text_section();
        layer.set_font(&font, 12.);
        layer.set_text_rendering_mode(printpdf::TextRenderingMode::FillStroke);
        layer.set_text_matrix(printpdf::TextMatrix::Raw([2., -1., 0., 3., 42., 71.]));
        layer.write_glyph(gid, '中', &font, 12.);
        layer.push(Op::ShowText {
            items: vec![
                TextItem::GlyphIds(vec![Codepoint::with_cid(gid, 0., "/".to_string())]),
                TextItem::Offset(-250.),
                TextItem::Text("/".to_string()),
            ],
        });
        layer.end_text_section();
        layer.restore_graphics_state();
        let image = easyofd_core::ImageObject::new(
            10.,
            20.,
            30.,
            40.,
            Vec::new(),
            easyofd_core::ImageFormat::Png,
        );
        layer.add_rgba_image(vec![255, 0, 0, 255, 0, 0, 255, 0], 2, 1, &image, 100.);
        super::defer_font_operations(&mut doc.0.borrow_mut());
        let original = doc.0.borrow().save(
            &PdfSaveOptions {
                optimize: false,
                subset_fonts: true,
                image_optimization: None,
                ..Default::default()
            },
            &mut Vec::new(),
        );
        let mut normalized = Vec::new();
        write_canonical_text_pdf(&original, &mut normalized).unwrap();
        let before = lopdf::Document::load_mem(&original).unwrap();
        let after = lopdf::Document::load_mem(&normalized).unwrap();
        let id = before.get_pages()[&1];
        assert_eq!(
            before.get_page_fonts(id).unwrap(),
            after.get_page_fonts(id).unwrap()
        );
        for (id, object) in &before.objects {
            if let Ok(stream) = object.as_stream() {
                if stream.dict.get(b"Length1").is_ok()
                    || stream.dict.get(b"Subtype").is_ok()
                    || stream.content.starts_with(b"/CIDInit")
                {
                    assert_eq!(
                        after.get_object(*id).unwrap(),
                        object,
                        "font bytes, ToUnicode, image pixels and SMask must remain unchanged"
                    );
                }
            }
        }
        let old = lopdf::content::Content::decode(&before.get_page_content(id)).unwrap();
        let new = lopdf::content::Content::decode(&after.get_page_content(id)).unwrap();
        assert_eq!(old.operations.len(), new.operations.len());
        let mut converted = 0;
        let mut multi_preserved = 0;
        for (old, new) in old.operations.iter().zip(&new.operations) {
            if old.operator == "TJ" && old.operands[0].as_array().unwrap().len() == 1 {
                assert_eq!(new.operator, "Tj");
                assert_eq!(
                    new.operands,
                    vec![old.operands[0].as_array().unwrap()[0].clone()]
                );
                converted += 1;
            } else {
                assert_eq!(old.operator, new.operator);
                assert_eq!(
                    old.operands, new.operands,
                    "matrices, rendering modes, colors and multiple-item TJ must be identical"
                );
                if old.operator == "TJ" {
                    multi_preserved += 1;
                }
            }
        }
        assert_eq!(converted, 1);
        assert_eq!(multi_preserved, 1);
        assert!(write_canonical_text_pdf(b"] TJ invalid pdf", &mut Vec::new()).is_err());
    }
}

#[cfg(test)]
mod compound_regressions {
    use super::{Mm, PdfDocument};
    fn content(path: &str) -> lopdf::content::Content {
        let (doc, page, layer) = PdfDocument::create("compound", Mm(100.), Mm(100.), "Layer");
        crate::render_path_to_pdf(
            &doc.get_page(page).get_layer(layer),
            &easyofd_core::PathObject::new(0., 0., path).fill_color(0x00ff_0000),
            100.,
        );
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        if let Some(dir) = std::env::var_os("EASYOFD_PDF_REVIEW_DIR") {
            let name = if path.matches('M').count() > 1 {
                "compound-holes.pdf"
            } else {
                "open-fill-stroke.pdf"
            };
            std::fs::write(std::path::Path::new(&dir).join(name), &bytes).unwrap();
        }
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        lopdf::content::Content::decode(&pdf.get_page_content(pdf.get_pages()[&1])).unwrap()
    }
    #[test]
    fn opposite_rings_share_one_nonzero_fill() {
        let content =
            content("M 10 10 L 40 10 L 40 40 L 10 40 Z M 20 20 L 20 30 L 30 30 L 30 20 Z");
        let fills: Vec<_> = content
            .operations
            .iter()
            .enumerate()
            .filter(|(_, op)| matches!(op.operator.as_str(), "f" | "f*" | "B" | "B*" | "b" | "b*"))
            .collect();
        assert_eq!(
            fills.len(),
            1,
            "compound rings must be filled together to preserve holes"
        );
        assert_eq!(fills[0].1.operator, "f", "fill uses nonzero winding");
        assert_eq!(
            content.operations[..fills[0].0]
                .iter()
                .filter(|op| op.operator == "m")
                .count(),
            2
        );
    }
    #[test]
    fn fill_does_not_close_an_open_stroke() {
        let content = content("M 10 10 L 30 10 L 30 30");
        let fill = content
            .operations
            .iter()
            .position(|op| op.operator == "f")
            .expect("fill must be separate from stroke");
        let stroke = content
            .operations
            .iter()
            .position(|op| op.operator == "S")
            .expect("open stroke uses S");
        assert!(stroke > fill);
        assert_eq!(
            content.operations[fill + 1..stroke]
                .iter()
                .filter(|op| op.operator == "m")
                .count(),
            1
        );
        assert_eq!(
            content.operations[fill + 1..stroke]
                .iter()
                .filter(|op| op.operator == "l")
                .count(),
            2
        );
        assert!(
            !content.operations[fill + 1..stroke]
                .iter()
                .any(|op| op.operator == "h"),
            "stroke must not acquire a closing edge"
        );
    }
}
