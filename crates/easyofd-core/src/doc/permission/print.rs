//! 打印权限。

/// 对应 Java: org.ofdrw.core.basicStructure.Print
///
/// 打印权限控制，限制可打印份数。
#[derive(Debug, Clone)]
pub struct Print {
    /// 是否允许打印。默认 true。
    pub printable: bool,
    /// 最大打印份数。None 表示不限制。
    pub copies: Option<u32>,
}

impl Print {
    /// 创建允许打印的权限。
    #[must_use]
    pub fn new() -> Self {
        Self {
            printable: true,
            copies: None,
        }
    }

    /// 创建禁止打印的权限。
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            printable: false,
            copies: None,
        }
    }

    /// 设置最大打印份数。
    #[must_use]
    pub fn with_copies(mut self, copies: u32) -> Self {
        self.copies = Some(copies);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let copies_attr = match self.copies {
            Some(c) => format!(" Copies=\"{c}\""),
            None => String::new(),
        };
        format!("<Print Printable=\"{}\"{copies_attr}/>", self.printable)
    }
}

impl Default for Print {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_new() {
        let p = Print::new();
        assert!(p.printable);
        assert!(p.copies.is_none());
        let p2 = Print::default();
        assert!(p2.printable);
    }

    #[test]
    fn test_print_disabled_and_xml() {
        let p = Print::disabled();
        let xml = p.to_xml_string();
        assert!(xml.contains("Printable=\"false\""));

        let p3 = Print::new().with_copies(3);
        let xml3 = p3.to_xml_string();
        assert!(xml3.contains("Copies=\"3\""));
        assert!(xml3.contains("Printable=\"true\""));
    }
}
