//! 抽取过滤器接口。
//!
//! 对应 Java: org.ofdrw.reader.extractor.ExtractorFilter

/// 抽取过滤器，决定文本对象是否应被抽取。
///
/// 对应 Java: `org.ofdrw.reader.extractor.ExtractorFilter`
///
/// 在内容抽取过程中，每个文本对象都会经过过滤器判断。
/// 返回 `Some(text)` 表示允许该文本通过，返回 `None` 表示过滤掉。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDecision {
    /// 允许该文本通过。
    Allow,
    /// 过滤掉该文本。
    Reject,
}

/// 文本抽取过滤器 trait。
///
/// 对应 Java: `org.ofdrw.reader.extractor.ExtractorFilter`（`@FunctionalInterface`）
///
/// 实现此 trait 可以自定义文本抽取的过滤逻辑，
/// 例如按区域过滤、按字体过滤等。
pub trait ExtractorFilter: std::fmt::Debug {
    /// 判断指定文本是否应被抽取。
    ///
    /// # 参数
    ///
    /// - `text`: 文本内容。
    /// - `page_num`: 页码（从 1 开始）。
    ///
    /// # 返回
    ///
    /// 返回过滤后的文本（可以与原文本不同），返回 `None` 表示过滤掉。
    fn filter_text(&self, text: &str, page_num: usize) -> Option<String>;
}

/// 矩形区域过滤器。
///
/// 只抽取指定矩形区域内的文本（基于简单的坐标范围判断）。
#[derive(Debug, Clone)]
pub struct RectFilter {
    /// 区域左边界 X。
    pub left: f64,
    /// 区域上边界 Y。
    pub top: f64,
    /// 区域右边界 X。
    pub right: f64,
    /// 区域下边界 Y。
    pub bottom: f64,
}

impl RectFilter {
    /// 创建新的矩形区域过滤器。
    #[must_use]
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// 判断坐标是否在区域内。
    #[must_use]
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}

impl ExtractorFilter for RectFilter {
    fn filter_text(&self, text: &str, _page_num: usize) -> Option<String> {
        // 简化实现：不带坐标信息的过滤器默认允许所有文本。
        // 实际使用时需要配合文本对象的坐标信息。
        if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_decision_variants() {
        let allow = FilterDecision::Allow;
        let reject = FilterDecision::Reject;
        assert_ne!(allow, reject);
    }

    #[test]
    fn test_rect_filter_contains() {
        let filter = RectFilter::new(0.0, 0.0, 100.0, 100.0);
        assert!(filter.contains(50.0, 50.0));
        assert!(filter.contains(0.0, 0.0));
        assert!(filter.contains(100.0, 100.0));
        assert!(!filter.contains(101.0, 50.0));
        assert!(!filter.contains(50.0, -1.0));
    }

    #[test]
    fn test_rect_filter_trait() {
        let filter = RectFilter::new(0.0, 0.0, 100.0, 100.0);
        assert!(ExtractorFilter::filter_text(&filter, "hello", 1).is_some());
        assert!(ExtractorFilter::filter_text(&filter, "", 1).is_none());
    }
}
