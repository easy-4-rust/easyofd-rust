//! 关键字位置。
//!
//! 对应 Java: org.ofdrw.reader.keyword.KeywordPosition

use easyofd_core::ST_Box;

/// 关键字位置，描述关键字在 OFD 文档中的定位信息。
///
/// 对应 Java: `org.ofdrw.reader.keyword.KeywordPosition`
#[derive(Debug, Clone)]
pub struct KeywordPosition {
    /// 关键字所在页码（从 1 开始）。
    pub page: usize,
    /// 关键字在页面中的矩形区域（毫米）。
    pub rect: ST_Box,
    /// 所属关键字文本。
    pub keyword: Option<String>,
}

impl KeywordPosition {
    /// 创建新的关键字位置。
    #[must_use]
    pub fn new(page: usize, rect: ST_Box) -> Self {
        Self {
            page,
            rect,
            keyword: None,
        }
    }

    /// 设置所属关键字。
    #[must_use]
    pub fn with_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keyword = Some(keyword.into());
        self
    }

    /// 获取矩形区域左上角 X 坐标。
    #[must_use]
    pub fn x(&self) -> f64 {
        self.rect.top_left_x
    }

    /// 获取矩形区域左上角 Y 坐标。
    #[must_use]
    pub fn y(&self) -> f64 {
        self.rect.top_left_y
    }

    /// 获取矩形区域宽度。
    #[must_use]
    pub fn width(&self) -> f64 {
        self.rect.width
    }

    /// 获取矩形区域高度。
    #[must_use]
    pub fn height(&self) -> f64 {
        self.rect.height
    }
}

impl std::fmt::Display for KeywordPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KeywordPosition{{page={}, rect=({}, {}, {}, {}), keyword={}}}",
            self.page,
            self.rect.top_left_x,
            self.rect.top_left_y,
            self.rect.width,
            self.rect.height,
            self.keyword.as_deref().unwrap_or("")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_position_new() {
        let rect = ST_Box::new(10.0, 20.0, 50.0, 12.0);
        let pos = KeywordPosition::new(1, rect);
        assert_eq!(pos.page, 1);
        assert!((pos.x() - 10.0).abs() < f64::EPSILON);
        assert!((pos.y() - 20.0).abs() < f64::EPSILON);
        assert!((pos.width() - 50.0).abs() < f64::EPSILON);
        assert!((pos.height() - 12.0).abs() < f64::EPSILON);
        assert!(pos.keyword.is_none());
    }

    #[test]
    fn test_keyword_position_with_keyword() {
        let rect = ST_Box::new(0.0, 0.0, 100.0, 20.0);
        let pos = KeywordPosition::new(3, rect).with_keyword("OFD");
        assert_eq!(pos.keyword.as_deref(), Some("OFD"));
    }

    #[test]
    fn test_keyword_position_display() {
        let rect = ST_Box::new(10.0, 20.0, 50.0, 12.0);
        let pos = KeywordPosition::new(1, rect).with_keyword("test");
        let s = format!("{pos}");
        assert!(s.contains("page=1"));
        assert!(s.contains("test"));
    }
}
