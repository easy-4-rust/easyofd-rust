//! 页面信息。
//!
//! 对应 Java: org.ofdrw.reader.PageInfo

use easyofd_core::{ST_Box, ST_ID, ST_Loc};

/// 页面信息，描述 OFD 文档中单个页面的元数据。
///
/// 对应 Java: `org.ofdrw.reader.PageInfo`
#[derive(Debug, Clone)]
pub struct PageInfo {
    /// 页面的物理大小（毫米）。
    pub size: ST_Box,
    /// 页面在 OFD 中的对象 ID。
    pub id: Option<ST_ID>,
    /// 页码，从 1 开始。
    pub index: usize,
    /// 页面的容器内绝对路径。
    pub page_abs_loc: Option<ST_Loc>,
    /// 页码目录文件的序号（Page_N 中的 N）。
    pub page_n: usize,
    /// 该页面引用的模板页面 ID 列表。
    pub template_ids: Vec<String>,
}

impl PageInfo {
    /// 创建新的页面信息。
    #[must_use]
    pub fn new(index: usize, size: ST_Box) -> Self {
        Self {
            size,
            id: None,
            index,
            page_abs_loc: None,
            page_n: index.saturating_sub(1),
            template_ids: Vec::new(),
        }
    }

    /// 设置页面 ID。
    #[must_use]
    pub fn with_id(mut self, id: ST_ID) -> Self {
        self.id = Some(id);
        self
    }

    /// 设置页面绝对路径。
    #[must_use]
    pub fn with_abs_loc(mut self, loc: ST_Loc) -> Self {
        self.page_abs_loc = Some(loc);
        self
    }

    /// 设置页码目录序号。
    #[must_use]
    pub fn with_page_n(mut self, n: usize) -> Self {
        self.page_n = n;
        self
    }

    /// 添加模板页面 ID。
    pub fn add_template(&mut self, template_id: String) {
        self.template_ids.push(template_id);
    }

    /// 获取页面宽度（毫米）。
    #[must_use]
    pub fn width(&self) -> f64 {
        self.size.width
    }

    /// 获取页面高度（毫米）。
    #[must_use]
    pub fn height(&self) -> f64 {
        self.size.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_info_new() {
        let size = ST_Box::new(0.0, 0.0, 210.0, 297.0);
        let info = PageInfo::new(1, size);
        assert_eq!(info.index, 1);
        assert!((info.width() - 210.0).abs() < f64::EPSILON);
        assert!((info.height() - 297.0).abs() < f64::EPSILON);
        assert!(info.id.is_none());
        assert!(info.template_ids.is_empty());
    }

    #[test]
    fn test_page_info_with_id() {
        let size = ST_Box::new(0.0, 0.0, 210.0, 297.0);
        let info = PageInfo::new(1, size).with_id(ST_ID::new(42).unwrap());
        assert_eq!(info.id.map(|id| id.get()), Some(42));
    }

    #[test]
    fn test_page_info_page_n_default() {
        let size = ST_Box::new(0.0, 0.0, 210.0, 297.0);
        let info = PageInfo::new(3, size);
        assert_eq!(info.page_n, 2);
    }

    #[test]
    fn test_page_info_add_template() {
        let size = ST_Box::new(0.0, 0.0, 210.0, 297.0);
        let mut info = PageInfo::new(1, size);
        info.add_template("tpl_0".into());
        info.add_template("tpl_1".into());
        assert_eq!(info.template_ids.len(), 2);
        assert_eq!(info.template_ids[0], "tpl_0");
    }

    #[test]
    fn test_page_info_with_page_n() {
        let size = ST_Box::new(0.0, 0.0, 210.0, 297.0);
        let info = PageInfo::new(1, size).with_page_n(5);
        assert_eq!(info.page_n, 5);
    }
}
