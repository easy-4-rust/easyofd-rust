//! 页面模板对象实体。
//!
//! 对应 Java: org.ofdrw.reader.model.TemplatePageEntity

/// 模板页面的 Z 序（绘制顺序）。
///
/// 对应 Java: `org.ofdrw.core.basicStructure.pageObj.layer.Type`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TemplateZOrder {
    /// 背景层（最先绘制）。
    Background,
    /// 主体层。
    Body,
    /// 前景层（最后绘制）。
    Foreground,
}

impl TemplateZOrder {
    /// 返回排序值（越小越先绘制）。
    #[must_use]
    pub fn order(&self) -> i32 {
        match self {
            Self::Background => 0,
            Self::Body => 1,
            Self::Foreground => 2,
        }
    }
}

/// 页面模板对象实体，描述一个模板页面的元数据和内容引用。
///
/// 对应 Java: `org.ofdrw.reader.model.TemplatePageEntity`
#[derive(Debug, Clone)]
pub struct TemplatePageEntity {
    /// 模板 ID。
    pub id: String,
    /// 模板页名称（可选）。
    pub name: Option<String>,
    /// 模板页面的 Z 序（绘制顺序）。
    pub z_order: TemplateZOrder,
    /// 指向模板页内容描述文件的路径。
    pub base_loc: String,
}

impl TemplatePageEntity {
    /// 创建新的模板页面实体。
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        z_order: TemplateZOrder,
        base_loc: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: None,
            z_order,
            base_loc: base_loc.into(),
        }
    }

    /// 设置模板页名称。
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置 Z 序。
    pub fn set_z_order(&mut self, z_order: TemplateZOrder) {
        self.z_order = z_order;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_z_order_ordering() {
        assert!(TemplateZOrder::Background.order() < TemplateZOrder::Body.order());
        assert!(TemplateZOrder::Body.order() < TemplateZOrder::Foreground.order());
    }

    #[test]
    fn test_template_page_entity_new() {
        let entity =
            TemplatePageEntity::new("tpl_0", TemplateZOrder::Body, "Pages/Page_0/Content.xml");
        assert_eq!(entity.id, "tpl_0");
        assert_eq!(entity.z_order, TemplateZOrder::Body);
        assert_eq!(entity.base_loc, "Pages/Page_0/Content.xml");
        assert!(entity.name.is_none());
    }

    #[test]
    fn test_template_page_entity_with_name() {
        let entity = TemplatePageEntity::new("tpl_1", TemplateZOrder::Background, "tpl.xml")
            .with_name("Header Template");
        assert_eq!(entity.name.as_deref(), Some("Header Template"));
    }

    #[test]
    fn test_template_page_entity_set_z_order() {
        let mut entity = TemplatePageEntity::new("tpl_0", TemplateZOrder::Body, "tpl.xml");
        entity.set_z_order(TemplateZOrder::Foreground);
        assert_eq!(entity.z_order, TemplateZOrder::Foreground);
    }
}
