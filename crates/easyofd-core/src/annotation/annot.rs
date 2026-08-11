//! 单个注释。

use std::fmt::Write;

use super::annot_type::AnnotType;
use super::appearance::Appearance;

/// 对应 Java: org.ofdrw.core.annotation.Annot
///
/// 表示一个单独的注释对象，包含注释的标识、创建者、类型、
/// 标志位、最后修改时间、位置和外观等信息。
#[derive(Debug, Clone)]
pub struct Annot {
    /// 注释 ID。
    pub id: String,
    /// 注释创建者。
    pub creator: Option<String>,
    /// 注释类型。
    pub annot_type: AnnotType,
    /// 注释标志位（如是否可打印、是否锁定等）。
    pub flags: u32,
    /// 最后修改日期（ISO 8601 格式字符串）。
    pub last_mod_date: Option<String>,
    /// 注释在页面上的矩形区域 [x, y, width, height]（单位 mm）。
    pub location: [f64; 4],
    /// 注释外观列表。
    pub appearances: Vec<Appearance>,
}

impl Annot {
    /// 创建一个新的注释。
    #[must_use]
    pub fn new(id: impl Into<String>, annot_type: AnnotType) -> Self {
        Self {
            id: id.into(),
            creator: None,
            annot_type,
            flags: 0,
            last_mod_date: None,
            location: [0.0, 0.0, 0.0, 0.0],
            appearances: Vec::new(),
        }
    }

    /// 设置创建者。
    #[must_use]
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    /// 设置标志位。
    #[must_use]
    pub fn flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    /// 设置最后修改日期。
    #[must_use]
    pub fn last_mod_date(mut self, date: impl Into<String>) -> Self {
        self.last_mod_date = Some(date.into());
        self
    }

    /// 设置位置。
    #[must_use]
    pub fn location(mut self, x: f64, y: f64, w: f64, h: f64) -> Self {
        self.location = [x, y, w, h];
        self
    }

    /// 添加外观。
    #[must_use]
    pub fn add_appearance(mut self, appearance: Appearance) -> Self {
        self.appearances.push(appearance);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!(
            r#"<ofd:Annot ID="{}" Type="{}" Flags="{}" "#,
            self.id,
            self.annot_type.as_str(),
            self.flags
        );

        if let Some(ref creator) = self.creator {
            let _ = write!(xml, r#"Creator="{creator}" "#);
        }

        if let Some(ref date) = self.last_mod_date {
            let _ = write!(xml, r#"LastModDate="{date}" "#);
        }

        let [x, y, w, h] = self.location;
        let _ = write!(xml, r#"Location="{x} {y} {w} {h}""#);

        if self.appearances.is_empty() {
            xml.push_str(" />");
        } else {
            xml.push('>');
            for app in &self.appearances {
                xml.push('\n');
                xml.push_str(&app.to_xml_string());
            }
            xml.push_str("\n</ofd:Annot>");
        }
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annot_new() {
        let a = Annot::new("ann1", AnnotType::Text);
        assert_eq!(a.id, "ann1");
        assert_eq!(a.annot_type, AnnotType::Text);
        assert_eq!(a.flags, 0);
        assert!(a.creator.is_none());
        assert!(a.last_mod_date.is_none());
        assert!((a.location[0]).abs() < f64::EPSILON);
        assert!((a.location[1]).abs() < f64::EPSILON);
        assert!((a.location[2]).abs() < f64::EPSILON);
        assert!((a.location[3]).abs() < f64::EPSILON);
        assert!(a.appearances.is_empty());
    }

    #[test]
    fn test_annot_builder() {
        let a = Annot::new("ann2", AnnotType::Highlight)
            .creator("user1")
            .flags(4)
            .last_mod_date("2025-01-01T00:00:00")
            .location(10.0, 20.0, 30.0, 40.0)
            .add_appearance(Appearance::new("app1", "Normal"));
        assert_eq!(a.creator.as_deref(), Some("user1"));
        assert_eq!(a.flags, 4);
        assert_eq!(a.last_mod_date.as_deref(), Some("2025-01-01T00:00:00"));
        assert!((a.location[0] - 10.0).abs() < f64::EPSILON);
        assert!((a.location[1] - 20.0).abs() < f64::EPSILON);
        assert!((a.location[2] - 30.0).abs() < f64::EPSILON);
        assert!((a.location[3] - 40.0).abs() < f64::EPSILON);
        assert_eq!(a.appearances.len(), 1);
    }

    #[test]
    fn test_annot_to_xml_string_basic() {
        let a = Annot::new("a1", AnnotType::Link);
        let xml = a.to_xml_string();
        assert!(xml.contains(r#"ID="a1""#));
        assert!(xml.contains(r#"Type="Link""#));
        assert!(xml.contains(r#"Flags="0""#));
        assert!(xml.contains(" />"));
    }

    #[test]
    fn test_annot_to_xml_string_full() {
        let a = Annot::new("a2", AnnotType::Stamp)
            .creator("admin")
            .last_mod_date("2025-06-01")
            .location(1.0, 2.0, 3.0, 4.0)
            .add_appearance(Appearance::new("app1", "Normal"));
        let xml = a.to_xml_string();
        assert!(xml.contains(r#"Creator="admin""#));
        assert!(xml.contains(r#"LastModDate="2025-06-01""#));
        assert!(xml.contains(r#"Location="1 2 3 4""#));
        assert!(xml.contains("ofd:Appearance"));
        assert!(xml.contains("</ofd:Annot>"));
    }
}
