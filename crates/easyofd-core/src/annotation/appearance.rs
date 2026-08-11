//! 注释静态外观。

use std::fmt::Write;

/// 对应 Java: org.ofdrw.core.annotation.Appearance
///
/// 注释的静态外观，描述注释在页面上的绘制方式。
#[derive(Debug, Clone)]
pub struct Appearance {
    /// 外观标识符。
    pub id: String,
    /// 外观类型（Normal / Rollover / Down）。
    pub appearance_type: String,
    /// 外观资源文件路径。
    pub resource_path: Option<String>,
}

impl Appearance {
    /// 创建一个新的外观对象。
    #[must_use]
    pub fn new(id: impl Into<String>, appearance_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            appearance_type: appearance_type.into(),
            resource_path: None,
        }
    }

    /// 设置资源路径。
    #[must_use]
    pub fn resource_path(mut self, path: impl Into<String>) -> Self {
        self.resource_path = Some(path.into());
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!(
            r#"<ofd:Appearance ID="{}" Type="{}""#,
            self.id, self.appearance_type
        );
        if let Some(ref path) = self.resource_path {
            let _ = write!(xml, r#" ResourcePath="{path}""#);
        }
        xml.push_str(" />");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appearance_new() {
        let a = Appearance::new("app1", "Normal");
        assert_eq!(a.id, "app1");
        assert_eq!(a.appearance_type, "Normal");
        assert!(a.resource_path.is_none());
    }

    #[test]
    fn test_appearance_builder() {
        let a = Appearance::new("app2", "Rollover").resource_path("/res/appearance.xml");
        assert_eq!(a.resource_path.as_deref(), Some("/res/appearance.xml"));
    }

    #[test]
    fn test_appearance_to_xml_string_basic() {
        let a = Appearance::new("app1", "Normal");
        let xml = a.to_xml_string();
        assert!(xml.contains(r#"ID="app1""#));
        assert!(xml.contains(r#"Type="Normal""#));
        assert!(!xml.contains("ResourcePath"));
    }

    #[test]
    fn test_appearance_to_xml_string_with_resource() {
        let a = Appearance::new("app1", "Normal").resource_path("/res/a.xml");
        let xml = a.to_xml_string();
        assert!(xml.contains(r#"ResourcePath="/res/a.xml""#));
    }

    #[test]
    fn test_appearance_clone_debug() {
        let a = Appearance::new("x", "Normal");
        let a2 = a.clone();
        assert_eq!(a2.id, "x");
        assert!(format!("{a:?}").contains("Appearance"));
    }
}
