//! 关键字抽取器。
//!
//! 对应 Java: org.ofdrw.reader.keyword.KeywordExtractor
//!
//! Java 版使用 AWT 字体引擎计算字符宽度来定位关键字。
//! Rust 版提供关键字搜索的结构和接口，实际的字体度量计算
//! 需要外部字体引擎支持。

use super::KeywordPosition;
use easyofd_core::OfdPage;

/// 每毫米的 point 单位（72pt / 25.4mm）。
const POINT_PER_MM: f64 = 72.0 / 25.4;

/// 关键字抽取器。
///
/// 对应 Java: `org.ofdrw.reader.keyword.KeywordExtractor`
///
/// 在 OFD 文档的文本内容中搜索关键字，返回匹配位置的页面和矩形区域。
///
/// 注意：Java 版使用 AWT `FontRenderContext` 计算精确的字符边界，
/// Rust 版使用简化的文本宽度估算。对于精确排版，需要集成外部字体引擎。
#[derive(Debug)]
pub struct KeywordExtractor;

impl KeywordExtractor {
    /// 获取关键字在文档中的位置列表。
    ///
    /// 对应 Java: `KeywordExtractor.getKeyWordPositionList(OFDReader, String)`
    ///
    /// 搜索所有页面的文本内容，返回包含关键字的矩形区域。
    ///
    /// # 参数
    ///
    /// - `pages`: 已解析的页面列表。
    /// - `keyword`: 要搜索的关键字。
    #[must_use]
    pub fn get_keyword_positions(pages: &[OfdPage], keyword: &str) -> Vec<KeywordPosition> {
        if keyword.is_empty() {
            return Vec::new();
        }

        let mut positions = Vec::new();
        for (page_idx, page) in pages.iter().enumerate() {
            let page_num = page_idx + 1;
            let page_positions = Self::search_page(page, page_num, keyword);
            positions.extend(page_positions);
        }
        positions
    }

    /// 在单个页面中搜索关键字。
    #[allow(clippy::cast_precision_loss)]
    fn search_page(page: &OfdPage, page_num: usize, keyword: &str) -> Vec<KeywordPosition> {
        use easyofd_core::ContentObject;

        let mut positions = Vec::new();
        for obj in &page.content {
            if let ContentObject::Text(text_obj) = obj {
                let text = &text_obj.text;
                // 查找所有匹配位置
                let mut start = 0;
                while let Some(idx) = text[start..].find(keyword) {
                    let match_start = start + idx;
                    // 估算关键字在文本中的位置
                    // 使用平均字符宽度近似（每个字符约 3mm 宽度作为默认值）
                    let char_width = 3.0;
                    let x = text_obj.x + (match_start as f64) * char_width;
                    let y = text_obj.y;
                    let kw_width = (keyword.len() as f64) * char_width;
                    let kw_height = text_obj.size / POINT_PER_MM;

                    let rect = easyofd_core::ST_Box::new(x, y, kw_width, kw_height);
                    positions.push(KeywordPosition::new(page_num, rect).with_keyword(keyword));
                    start = match_start + keyword.len();
                }
            }
        }
        positions
    }

    /// 获取关键字坐标列表（带字体度量的精确版本）。
    ///
    /// 对应 Java: `KeywordExtractor.getKeyWordPositionList(OFDReader, String)`
    ///
    /// 此方法接受字体大小映射表，用于更精确地计算字符宽度。
    ///
    /// # 参数
    ///
    /// - `pages`: 已解析的页面列表。
    /// - `keyword`: 要搜索的关键字。
    /// - `font_sizes`: 字体大小映射（字体 ID -> 字号，单位毫米）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_keyword_positions_with_fonts(
        pages: &[OfdPage],
        keyword: &str,
        font_sizes: &std::collections::HashMap<String, f64>,
    ) -> Vec<KeywordPosition> {
        if keyword.is_empty() {
            return Vec::new();
        }

        let mut positions = Vec::new();
        for (page_idx, page) in pages.iter().enumerate() {
            let page_num = page_idx + 1;
            for obj in &page.content {
                if let easyofd_core::ContentObject::Text(text_obj) = obj {
                    let text = &text_obj.text;
                    let char_width = font_sizes
                        .get(&text_obj.font)
                        .map_or(3.0, |size| size / POINT_PER_MM * 0.6);

                    let mut start = 0;
                    while let Some(idx) = text[start..].find(keyword) {
                        let match_start = start + idx;
                        let x = text_obj.x + (match_start as f64) * char_width;
                        let y = text_obj.y;
                        let kw_width = (keyword.len() as f64) * char_width;
                        let kw_height = text_obj.size / POINT_PER_MM;

                        let rect = easyofd_core::ST_Box::new(x, y, kw_width, kw_height);
                        positions.push(KeywordPosition::new(page_num, rect).with_keyword(keyword));
                        start = match_start + keyword.len();
                    }
                }
            }
        }
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::TextObject;

    fn make_page_with_text(text: &str) -> OfdPage {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, text).size(10.0));
        page
    }

    #[test]
    fn test_keyword_extractor_single_match() {
        let pages = vec![make_page_with_text("Hello World OFD Test")];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "OFD");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].page, 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("OFD"));
    }

    #[test]
    fn test_keyword_extractor_multiple_matches() {
        let pages = vec![make_page_with_text("OFD is an OFD format")];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "OFD");
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn test_keyword_extractor_no_match() {
        let pages = vec![make_page_with_text("Hello World")];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "OFD");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_keyword_extractor_empty_keyword() {
        let pages = vec![make_page_with_text("Hello")];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_keyword_extractor_multiple_pages() {
        let pages = vec![
            make_page_with_text("Page 1"),
            make_page_with_text("Page 2 OFD"),
        ];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "OFD");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].page, 2);
    }

    #[test]
    fn test_keyword_extractor_with_fonts() {
        let pages = vec![make_page_with_text("Test OFD keyword")];
        let mut font_sizes = std::collections::HashMap::new();
        font_sizes.insert("SimHei".to_string(), 12.0);
        let positions =
            KeywordExtractor::get_keyword_positions_with_fonts(&pages, "OFD", &font_sizes);
        assert_eq!(positions.len(), 1);
    }

    #[test]
    fn test_point_per_mm_constant() {
        assert!((POINT_PER_MM - 2.8346).abs() < 0.01);
    }
}
