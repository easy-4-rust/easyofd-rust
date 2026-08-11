//! CT_Composite 复合对象容器。
//!
//! 对应 GB/T 33190-2016 第 13.6 节中的 CT_Composite 类型。
//! 复合对象是页面对象的容器，可以包含多个子对象（文本、图像、路径等），
//! 实现对象的分组和整体变换。

/// 对应 Java: org.ofdrw.core.compositeObj.CT_Composite
///
/// 复合对象容器。用于将多个页面对象组合为一个逻辑单元，
/// 支持统一的位置变换、裁剪等操作。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_Composite {
    /// 对象 ID，在页面内唯一。
    pub id: u32,
    /// 对象边界框，格式为 "x y width height"（单位 mm）。
    pub boundary: String,
    /// 对象名称（可选），用于标识复合对象。
    pub name: Option<String>,
    /// 可见性。true 表示可见（默认），false 表示隐藏。
    pub visible: bool,
    /// 子对象列表，按绘制顺序排列。
    pub children: Vec<CompositeChild>,
}

/// 复合对象中的子对象。
///
/// 子对象可以是文本、路径或其他复合对象。
#[derive(Debug, Clone)]
pub enum CompositeChild {
    /// 文本子对象（简化表示）。
    Text {
        /// 文本内容。
        content: String,
        /// X 坐标（mm）。
        x: f64,
        /// Y 坐标（mm）。
        y: f64,
        /// 字号（pt）。
        font_size: f64,
    },
    /// 路径子对象（简化表示）。
    Path {
        /// 路径数据（SVG 风格）。
        data: String,
        /// 描边颜色 RGB hex。
        stroke_color: u32,
    },
    /// 嵌套复合对象。
    Composite(CT_Composite),
}

impl CT_Composite {
    /// 创建新的复合对象容器。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            name: None,
            visible: true,
            children: Vec::new(),
        }
    }

    /// 设置对象名称。
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置可见性。
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// 添加文本子对象。
    pub fn add_text(&mut self, content: impl Into<String>, x: f64, y: f64, font_size: f64) {
        self.children.push(CompositeChild::Text {
            content: content.into(),
            x,
            y,
            font_size,
        });
    }

    /// 添加路径子对象。
    pub fn add_path(&mut self, data: impl Into<String>, stroke_color: u32) {
        self.children.push(CompositeChild::Path {
            data: data.into(),
            stroke_color,
        });
    }

    /// 添加嵌套复合对象。
    pub fn add_composite(&mut self, child: CT_Composite) {
        self.children.push(CompositeChild::Composite(child));
    }

    /// 子对象数量。
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = format!(
            "<ofd:CT_Composite ID=\"{}\" Boundary=\"{}\"",
            self.id, self.boundary
        );
        if let Some(ref name) = self.name {
            write!(xml, " Name=\"{name}\"").unwrap();
        }
        if !self.visible {
            xml.push_str(" Visible=\"false\"");
        }
        xml.push_str(">\n");
        for child in &self.children {
            xml.push_str(&child.to_xml_string());
        }
        xml.push_str("</ofd:CT_Composite>\n");
        xml
    }
}

impl CompositeChild {
    /// 序列化子对象为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        match self {
            Self::Text {
                content,
                x,
                y,
                font_size,
            } => {
                format!(
                    "  <ofd:TextObject X=\"{x}\" Y=\"{y}\" FontSize=\"{font_size}\">\
                     {content}</ofd:TextObject>\n"
                )
            }
            Self::Path { data, stroke_color } => {
                format!(
                    "  <ofd:PathObject StrokeColor=\"{stroke_color}\">\
                     <ofd:AbbreviatedData>{data}</ofd:AbbreviatedData>\
                     </ofd:PathObject>\n"
                )
            }
            Self::Composite(inner) => {
                // Indent nested composite.
                use std::fmt::Write;
                let inner_xml = inner.to_xml_string();
                let mut out = String::new();
                for line in inner_xml.lines() {
                    writeln!(out, "  {line}").unwrap();
                }
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_composite_new() {
        let c = CT_Composite::new(1, "0 0 100 100");
        assert_eq!(c.id, 1);
        assert_eq!(c.boundary, "0 0 100 100");
        assert!(c.name.is_none());
        assert!(c.visible);
        assert!(c.children.is_empty());
    }

    #[test]
    fn test_ct_composite_builder() {
        let c = CT_Composite::new(2, "10 20 50 50")
            .name("group1")
            .visible(false);
        assert_eq!(c.name.as_deref(), Some("group1"));
        assert!(!c.visible);
    }

    #[test]
    fn test_ct_composite_add_children() {
        let mut c = CT_Composite::new(3, "0 0 200 200");
        c.add_text("hello", 10.0, 20.0, 12.0);
        c.add_path("M0 0L10 10", 0x00_0000);
        assert_eq!(c.child_count(), 2);
    }

    #[test]
    fn test_ct_composite_nested() {
        let inner = CT_Composite::new(4, "5 5 10 10");
        let mut outer = CT_Composite::new(5, "0 0 100 100");
        outer.add_composite(inner);
        assert_eq!(outer.child_count(), 1);
    }

    #[test]
    fn test_ct_composite_to_xml_string() {
        let c = CT_Composite::new(10, "0 0 50 50").name("myGroup");
        let xml = c.to_xml_string();
        assert!(xml.contains("ID=\"10\""));
        assert!(xml.contains("Boundary=\"0 0 50 50\""));
        assert!(xml.contains("Name=\"myGroup\""));
        assert!(xml.contains("<ofd:CT_Composite"));
        assert!(xml.contains("</ofd:CT_Composite>"));
    }

    #[test]
    fn test_ct_composite_to_xml_with_children() {
        let mut c = CT_Composite::new(11, "0 0 100 100");
        c.add_text("test", 1.0, 2.0, 14.0);
        c.add_path("M0 0", 0xFF_0000);
        let xml = c.to_xml_string();
        assert!(xml.contains("ofd:TextObject"));
        assert!(xml.contains("ofd:PathObject"));
        assert!(xml.contains("test"));
    }

    #[test]
    fn test_ct_composite_to_xml_hidden() {
        let c = CT_Composite::new(12, "0 0 10 10").visible(false);
        let xml = c.to_xml_string();
        assert!(xml.contains("Visible=\"false\""));
    }

    #[test]
    fn test_ct_composite_clone_debug() {
        let c = CT_Composite::new(1, "0 0 1 1");
        let c2 = c.clone();
        assert_eq!(c2.id, 1);
        assert!(format!("{c:?}").contains("CT_Composite"));
    }
}
