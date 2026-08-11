//! CT_TemplatePage 模板页。

/// 对应 Java: org.ofdrw.core.basicStructure.pageObj.CT_TemplatePage
///
/// 模板页描述，定义可复用的页面模板。
/// 对应 GB/T 33190-2016 图 14。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_TemplatePage {
    /// 模板页 ID。
    pub id: u32,
    /// 模板页名称。
    pub name: Option<String>,
    /// Z 序（绘制顺序）。
    pub z_order: Option<TemplateZOrder>,
    /// 模板页文件路径。
    pub base_loc: Option<String>,
}

/// 模板页 Z 序枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateZOrder {
    /// 背景。
    Back,
    /// 前景。
    Front,
}

impl TemplateZOrder {
    /// 转为 OFD XML 属性值。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Back => "Back",
            Self::Front => "Front",
        }
    }
}

impl std::fmt::Display for TemplateZOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CT_TemplatePage {
    /// 创建新的模板页。
    #[must_use]
    pub fn new(id: u32) -> Self {
        Self {
            id,
            name: None,
            z_order: None,
            base_loc: None,
        }
    }

    /// 设置模板页名称。
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置 Z 序。
    #[must_use]
    pub fn z_order(mut self, z_order: TemplateZOrder) -> Self {
        self.z_order = Some(z_order);
        self
    }

    /// 设置模板页文件路径。
    #[must_use]
    pub fn base_loc(mut self, loc: impl Into<String>) -> Self {
        self.base_loc = Some(loc.into());
        self
    }

    /// 获取模板页 ID。
    #[must_use]
    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// 获取模板页名称。
    #[must_use]
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 获取 Z 序。
    #[must_use]
    pub fn get_z_order(&self) -> Option<TemplateZOrder> {
        self.z_order
    }

    /// 获取模板页文件路径。
    #[must_use]
    pub fn get_base_loc(&self) -> Option<&str> {
        self.base_loc.as_deref()
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = format!("<ofd:TemplatePage ID=\"{}\"", self.id);
        if let Some(ref name) = self.name {
            write!(xml, " TemplatePageName=\"{name}\"").unwrap();
        }
        if let Some(zo) = self.z_order {
            write!(xml, " ZOrder=\"{}\"", zo.as_str()).unwrap();
        }
        if let Some(ref loc) = self.base_loc {
            write!(xml, " BaseLoc=\"{loc}\"").unwrap();
        }
        xml.push_str(" />");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_template_page_new() {
        let tpl = CT_TemplatePage::new(1);
        assert_eq!(tpl.id, 1);
        assert!(tpl.name.is_none());
        assert!(tpl.z_order.is_none());
        assert!(tpl.base_loc.is_none());
    }

    #[test]
    fn test_ct_template_page_builder() {
        let tpl = CT_TemplatePage::new(5)
            .name("tpl1")
            .z_order(TemplateZOrder::Back)
            .base_loc("Tpl_5.xml");
        assert_eq!(tpl.get_id(), 5);
        assert_eq!(tpl.get_name(), Some("tpl1"));
        assert_eq!(tpl.get_z_order(), Some(TemplateZOrder::Back));
        assert_eq!(tpl.get_base_loc(), Some("Tpl_5.xml"));
    }

    #[test]
    fn test_template_z_order_display() {
        assert_eq!(TemplateZOrder::Back.to_string(), "Back");
        assert_eq!(TemplateZOrder::Front.to_string(), "Front");
    }

    #[test]
    fn test_template_z_order_as_str() {
        assert_eq!(TemplateZOrder::Back.as_str(), "Back");
        assert_eq!(TemplateZOrder::Front.as_str(), "Front");
    }

    #[test]
    fn test_ct_template_page_to_xml_minimal() {
        let tpl = CT_TemplatePage::new(1);
        let xml = tpl.to_xml_string();
        assert!(xml.contains("ID=\"1\""));
        assert!(xml.contains("<ofd:TemplatePage"));
        assert!(xml.ends_with(" />"));
    }

    #[test]
    fn test_ct_template_page_to_xml_full() {
        let tpl = CT_TemplatePage::new(10)
            .name("myTpl")
            .z_order(TemplateZOrder::Front)
            .base_loc("Tpl_10.xml");
        let xml = tpl.to_xml_string();
        assert!(xml.contains("ID=\"10\""));
        assert!(xml.contains("TemplatePageName=\"myTpl\""));
        assert!(xml.contains("ZOrder=\"Front\""));
        assert!(xml.contains("BaseLoc=\"Tpl_10.xml\""));
    }

    #[test]
    fn test_ct_template_page_clone_debug() {
        let tpl = CT_TemplatePage::new(1).name("x");
        let tpl2 = tpl.clone();
        assert_eq!(tpl2.get_name(), Some("x"));
        assert!(format!("{tpl:?}").contains("CT_TemplatePage"));
    }
}
