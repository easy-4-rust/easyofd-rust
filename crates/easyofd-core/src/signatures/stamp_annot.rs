//! 签章注释（StampAnnot）。
//!
//! 对应 Java: org.ofdrw.core.signatures.appearance.StampAnnot
//!
//! 一个数字签名可以跟一个或多个外观描述关联，也可以不关联任何外观。
//! GB/T 33190 第 18.2.3 节 图 88 表 69。

use std::fmt::Write;

use crate::basic_type::{ST_Box, ST_RefID};

/// 签章注释。
///
/// 描述数字签名在页面上的外观位置和裁剪区域。
/// 推荐使用 `sNNN` 编码方式，NNN 从 1 开始。
#[derive(Debug, Clone)]
pub struct StampAnnot {
    /// 签章注释标识（必选）。
    /// 推荐使用 `sNNN` 的编码方式，NNN 从 1 开始。
    pub id: String,
    /// 引用外观注释所在页面的标识符（必选）。
    pub page_ref: ST_RefID,
    /// 签章注释的外观边框位置（必选），用于页面内定位。
    pub boundary: ST_Box,
    /// 签章注释的外观裁剪设置（可选）。
    pub clip: Option<ST_Box>,
}

impl StampAnnot {
    /// 创建新的签章注释。
    #[must_use]
    pub fn new(id: impl Into<String>, page_ref: ST_RefID, boundary: ST_Box) -> Self {
        Self {
            id: id.into(),
            page_ref,
            boundary,
            clip: None,
        }
    }

    /// 设置外观裁剪区域。
    #[must_use]
    pub fn clip(mut self, clip: ST_Box) -> Self {
        self.clip = Some(clip);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!(
            r#"<ofd:StampAnnot ID="{}" PageRef="{}" Boundary="{}""#,
            self.id,
            self.page_ref.to_xml_string(),
            self.boundary.to_xml_string()
        );
        if let Some(ref clip) = self.clip {
            let _ = write!(xml, r#" Clip="{}""#, clip.to_xml_string());
        }
        xml.push_str(" />");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stamp_annot_new() {
        let sa = StampAnnot::new("s1", ST_RefID::new(1), ST_Box::new(10.0, 20.0, 50.0, 80.0));
        assert_eq!(sa.id, "s1");
        assert_eq!(sa.page_ref.to_xml_string(), "1");
        assert!(sa.clip.is_none());
    }

    #[test]
    fn test_stamp_annot_with_clip() {
        let sa = StampAnnot::new("s2", ST_RefID::new(3), ST_Box::new(0.0, 0.0, 100.0, 100.0))
            .clip(ST_Box::new(5.0, 5.0, 90.0, 90.0));
        assert!(sa.clip.is_some());
        let clip = sa.clip.unwrap();
        assert!((clip.top_left_x - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_stamp_annot_xml_without_clip() {
        let sa = StampAnnot::new("s1", ST_RefID::new(1), ST_Box::new(10.0, 20.0, 50.0, 80.0));
        let xml = sa.to_xml_string();
        assert!(xml.contains(r#"ID="s1""#));
        assert!(xml.contains("PageRef=\"1\""));
        assert!(xml.contains("Boundary=\"10 20 50 80\""));
        assert!(!xml.contains("Clip"));
        assert!(xml.contains("/>"));
    }

    #[test]
    fn test_stamp_annot_xml_with_clip() {
        let sa = StampAnnot::new("s3", ST_RefID::new(2), ST_Box::new(0.0, 0.0, 200.0, 200.0))
            .clip(ST_Box::new(10.0, 10.0, 180.0, 180.0));
        let xml = sa.to_xml_string();
        assert!(xml.contains(r#"ID="s3""#));
        assert!(xml.contains("Clip=\"10 10 180 180\""));
    }
}
