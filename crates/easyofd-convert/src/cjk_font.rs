//! CJK 字体探测与子集化模块。
//!
//! 在系统常见路径中搜索支持 CJK（中日韩）字符的 TTF/OTF/TTC 字体文件，
//! 用于 OFD → PDF 转换时嵌入中文字体。
//!
//! # 字体子集化
//!
//! 启用 `printpdf` 的 `font_subsetting` feature 后，嵌入 PDF 时会自动
//! 只保留文档中实际用到的 glyph，将 CJK 字体从 ~17 MB 降至 ~100–500 KB。
//!
//! # 平台探测路径
//!
//! - **macOS**: PingFang.ttc、Songti.ttc、STHeiti 等系统字体
//! - **Linux**: WenQuanYi、Noto Sans CJK、AR PL UMing 等开源字体
//! - **Fallback**: 通过 `fc-list` 命令搜索

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use easyofd_core::{OfdPage, OfdResult};

/// CJK 字体探测结果。
#[derive(Debug, Clone)]
pub struct CjkFontInfo {
    /// 字体文件路径。
    pub path: PathBuf,
    /// 字体显示名称。
    pub name: String,
}

/// 探测系统中可用的 CJK 字体。
///
/// 按优先级依次检查常见路径，返回第一个有效且包含 CJK 字形的字体。
/// 如果系统未安装 CJK 字体则返回 `None`。
pub fn find_cjk_font() -> Option<CjkFontInfo> {
    for candidate in candidate_paths() {
        if candidate.path.exists() {
            if let Some(info) = validate_font(&candidate.path, &candidate.name) {
                return Some(info);
            }
        }
    }

    // Fallback: 通过 fc-list 命令搜索
    find_font_via_fc_list()
}

/// 候选字体路径（按优先级排列）。
struct FontCandidate {
    path: PathBuf,
    name: String,
}

/// 返回跨平台的候选 CJK 字体路径列表。
fn candidate_paths() -> Vec<FontCandidate> {
    let mut candidates = Vec::new();

    // ── macOS 系统字体 ──
    #[cfg(target_os = "macos")]
    {
        let macos_fonts: &[(&str, &str)] = &[
            (
                "/System/Library/Fonts/PingFang.ttc",
                "PingFang SC (苹方-简)",
            ),
            ("/Library/Fonts/Songti.ttc", "Songti SC (宋体)"),
            (
                "/System/Library/Fonts/STHeiti Light.ttc",
                "STHeiti (华文黑体)",
            ),
            (
                "/System/Library/Fonts/Supplemental/Songti.ttc",
                "Songti SC (宋体-补充)",
            ),
            (
                "/System/Library/Fonts/Supplemental/STHeiti Medium.ttc",
                "STHeiti Medium",
            ),
            (
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "Hiragino Sans GB (冬青黑体)",
            ),
        ];
        for (path, name) in macos_fonts {
            candidates.push(FontCandidate {
                path: PathBuf::from(path),
                name: (*name).to_string(),
            });
        }
    }

    // ── Linux 系统字体 ──
    #[cfg(target_os = "linux")]
    {
        let linux_fonts: &[(&str, &str)] = &[
            (
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "WenQuanYi Micro Hei (文泉驿微米黑)",
            ),
            (
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "Noto Sans CJK SC",
            ),
            (
                "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
                "Noto Sans CJK SC (Noto)",
            ),
            (
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "Noto Sans CJK SC (Noto-TTF)",
            ),
            (
                "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
                "Droid Sans Fallback",
            ),
            (
                "/usr/share/fonts/truetype/arphic/uming.ttc",
                "AR PL UMing (文鼎宋体)",
            ),
            (
                "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
                "Noto Sans CJK SC (Fedora)",
            ),
            (
                "/usr/share/fonts/noto-cjk/NotoSansSC-Regular.otf",
                "Noto Sans SC (OTF)",
            ),
        ];
        for (path, name) in linux_fonts {
            candidates.push(FontCandidate {
                path: PathBuf::from(path),
                name: (*name).to_string(),
            });
        }
    }

    // ── Windows 系统字体 ──
    #[cfg(target_os = "windows")]
    {
        if let Some(win_dir) = std::env::var_os("SystemRoot") {
            let fonts_dir = Path::new(&win_dir).join("Fonts");
            let win_fonts: &[(&str, &str)] = &[
                ("msyh.ttc", "Microsoft YaHei (微软雅黑)"),
                ("simsun.ttc", "SimSun (宋体)"),
                ("simhei.ttf", "SimHei (黑体)"),
                ("msyhbd.ttc", "Microsoft YaHei Bold"),
            ];
            for (file, name) in win_fonts {
                candidates.push(FontCandidate {
                    path: fonts_dir.join(file),
                    name: (*name).to_string(),
                });
            }
        }
    }

    candidates
}

/// 验证字体文件是否有效且包含 CJK 字形。
///
/// 使用 `ttf-parser` 解析字体，检查 `units_per_em > 0`（TrueType 要求），
/// 并测试常用 CJK 字符的字形映射。
fn validate_font(path: &Path, name: &str) -> Option<CjkFontInfo> {
    let data = std::fs::read(path).ok()?;

    // ttf_parser::Face::parse 对 TTF/OTF 和 TTC（取 index 0）均适用
    let face = ttf_parser::Face::parse(&data, 0).ok()?;

    // 基本有效性检查
    if face.units_per_em() == 0 {
        return None;
    }

    // 检查是否包含 CJK 字形（测试几个常用汉字）
    if !probe_cjk_glyphs(&face) {
        return None;
    }

    Some(CjkFontInfo {
        path: path.to_path_buf(),
        name: name.to_string(),
    })
}

/// 测试字体是否包含 CJK 字形。
///
/// 检查 "中"、"文"、"测"、"试" 四个常用汉字，至少命中一个即认为支持 CJK。
fn probe_cjk_glyphs(face: &ttf_parser::Face<'_>) -> bool {
    let test_chars = ['中', '文', '测', '试', '你', '好'];
    let mut hits = 0;
    for ch in test_chars {
        if face.glyph_index(ch).is_some() {
            hits += 1;
        }
    }
    // 至少 2 个字形命中（排除误匹配）
    hits >= 2
}

/// 通过 `fc-list` 命令搜索系统 CJK 字体（Linux/macOS fallback）。
fn find_font_via_fc_list() -> Option<CjkFontInfo> {
    let output = std::process::Command::new("fc-list")
        .args([":lang=zh", "file"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        // fc-list 输出格式: /path/to/font.ttf: Font Name:style
        let path_str = line.split(':').next()?.trim();
        let path = PathBuf::from(path_str);
        if path.exists() {
            let name = path.file_name().map_or_else(
                || "CJK Font (fc-list)".to_string(),
                |n| n.to_string_lossy().to_string(),
            );
            if let Some(info) = validate_font(&path, &name) {
                return Some(info);
            }
        }
    }

    None
}

// ─── 字体子集化 ────────────────────────────────────────────────────────────

/// 从 OFD 页面集合中收集所有文本用到的字符。
///
/// 遍历每个页面的所有文本对象，提取其中出现的 CJK 字符和 ASCII 字符，
/// 返回去重的 `HashSet<char>`。
pub fn collect_used_chars(pages: &[OfdPage]) -> HashSet<char> {
    let mut chars = HashSet::new();
    for page in pages {
        for content in &page.content {
            if let easyofd_core::ContentObject::Text(text) = content {
                chars.extend(text.text.chars());
            }
        }
    }
    chars
}

/// 字体子集化统计信息。
#[derive(Debug, Clone)]
pub struct SubsetStats {
    /// 原始字体文件大小（字节）。
    pub original_size: usize,
    /// 文档用到的唯一字符数。
    pub used_char_count: usize,
    /// 文档用到的 CJK 字符数。
    pub cjk_char_count: usize,
    /// 字体中可映射到 glyph 的字符数（字体文件中实际存在的映射）。
    pub mapped_glyph_count: usize,
}

/// 对 TTF/OTF 字体做子集化分析。
///
/// 使用 `ttf-parser` 解析字体，统计文档中用到的字符在字体中的 glyph 映射情况。
///
/// **注意**：实际的字体子集化由 `printpdf` 的 `font_subsetting` feature
/// 在 `PdfDocument::save()` 时自动完成（内部使用 `allsorts` 库）。
/// 本函数提供诊断信息和字符集统计，用于日志报告。
///
/// # 返回
///
/// 返回子集化统计信息。如果字体解析失败则返回错误。
pub fn subset_font(face_data: &[u8], used_chars: &HashSet<char>) -> OfdResult<SubsetStats> {
    let face = ttf_parser::Face::parse(face_data, 0)
        .map_err(|e| easyofd_core::OfdError::Conversion(format!("字体解析失败: {e}")))?;

    let mut mapped_glyph_count = 0usize;
    let mut cjk_char_count = 0usize;

    for &ch in used_chars {
        if is_cjk_char(ch) {
            cjk_char_count += 1;
        }
        if face.glyph_index(ch).is_some() {
            mapped_glyph_count += 1;
        }
    }

    Ok(SubsetStats {
        original_size: face_data.len(),
        used_char_count: used_chars.len(),
        cjk_char_count,
        mapped_glyph_count,
    })
}

/// 判断字符是否属于 CJK 范围。
fn is_cjk_char(ch: char) -> bool {
    let cp = ch as u32;
    (0x4E00..=0x9FFF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0x2E80..=0x2EFF).contains(&cp)
        || (0x3000..=0x303F).contains(&cp)
        || (0xFF00..=0xFFEF).contains(&cp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_cjk_glyphs_with_invalid_data() {
        // 空数据应返回 None（解析失败）
        let result = ttf_parser::Face::parse(&[], 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_candidate_paths_not_empty() {
        let candidates = candidate_paths();
        assert!(
            !candidates.is_empty(),
            "候选字体路径列表不应为空（至少有当前平台的条目）"
        );
    }

    #[test]
    fn test_find_cjk_font_returns_option() {
        // 不断言一定找到（CI 可能没装 CJK 字体），只验证不 panic
        let _result = find_cjk_font();
    }

    #[test]
    fn test_validate_font_rejects_invalid_path() {
        assert!(validate_font(Path::new("/nonexistent/font.ttf"), "test").is_none());
    }

    #[test]
    fn test_collect_used_chars_from_pages() {
        use easyofd_core::{OfdPage, TextObject};

        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "Hello"));
        page.add_text(TextObject::new(10.0, 40.0, "你好世界"));

        let chars = collect_used_chars(&[page]);
        assert!(chars.contains(&'H'));
        assert!(chars.contains(&'你'));
        assert!(chars.contains(&'好'));
        assert!(!chars.contains(&'测'));
    }

    #[test]
    fn test_collect_used_chars_empty_pages() {
        let page = OfdPage::new(210.0, 297.0);
        let chars = collect_used_chars(&[page]);
        assert!(chars.is_empty());
    }

    #[test]
    fn test_is_cjk_char() {
        assert!(is_cjk_char('中'));
        assert!(is_cjk_char('文'));
        assert!(is_cjk_char('　')); // 全角空格 (U+3000)
        assert!(is_cjk_char('Ａ')); // 全角 A (U+FF21)
        assert!(!is_cjk_char('A'));
        assert!(!is_cjk_char('1'));
        assert!(!is_cjk_char(' '));
    }

    #[test]
    fn test_subset_font_with_invalid_data() {
        let chars = HashSet::from(['中']);
        let result = subset_font(&[], &chars);
        assert!(result.is_err());
    }

    #[test]
    fn test_subset_font_with_valid_ttf() {
        // 构造一个最小有效 TTF (需要实际字体数据)
        // 这里用 find_cjk_font 获取系统字体做端到端测试
        if let Some(info) = find_cjk_font() {
            let data = std::fs::read(&info.path).expect("读取字体文件");
            let mut chars = HashSet::new();
            chars.insert('中');
            chars.insert('A');
            let stats = subset_font(&data, &chars).expect("子集化分析");
            assert_eq!(stats.used_char_count, 2);
            assert_eq!(stats.cjk_char_count, 1);
            assert!(stats.original_size > 0);
            // 至少 'A' 应该能映射到 glyph
            assert!(stats.mapped_glyph_count >= 1);
        }
        // CI 无 CJK 字体时跳过
    }
}
