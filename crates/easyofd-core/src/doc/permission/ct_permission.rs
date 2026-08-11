//! 权限容器。

use super::{Print, ValidPeriod};

/// 对应 Java: org.ofdrw.core.basicStructure.CT_Permission
///
/// 文档权限控制容器，定义文档的打印权限和有效期。
#[derive(Debug, Clone)]
pub struct CtPermission {
    /// 是否可打印。默认 true。
    pub printable: bool,
    /// 是否可编辑。默认 true。
    pub editable: bool,
    /// 是否可注释。默认 true。
    pub annotatable: bool,
    /// 打印权限详情。可选。
    pub print: Option<Print>,
    /// 有效期。可选。
    pub valid_period: Option<ValidPeriod>,
}

impl CtPermission {
    /// 创建默认权限（全部允许）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            printable: true,
            editable: true,
            annotatable: true,
            print: None,
            valid_period: None,
        }
    }

    /// 设置打印权限。
    #[must_use]
    pub fn with_print(mut self, print: Print) -> Self {
        self.print = Some(print);
        self
    }

    /// 设置有效期。
    #[must_use]
    pub fn with_valid_period(mut self, period: ValidPeriod) -> Self {
        self.valid_period = Some(period);
        self
    }

    /// 禁止编辑。
    #[must_use]
    pub fn read_only(mut self) -> Self {
        self.editable = false;
        self.annotatable = false;
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut inner = String::new();
        if let Some(p) = &self.print {
            inner.push_str(&p.to_xml_string());
        }
        if let Some(vp) = &self.valid_period {
            inner.push_str(&vp.to_xml_string());
        }
        format!(
            "<CT_Permission Printable=\"{}\" Editable=\"{}\" Annotatable=\"{}\">{inner}</CT_Permission>",
            self.printable, self.editable, self.annotatable
        )
    }
}

impl Default for CtPermission {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_permission_new() {
        let p = CtPermission::new();
        assert!(p.printable);
        assert!(p.editable);
        assert!(p.annotatable);
        assert!(p.print.is_none());
        assert!(p.valid_period.is_none());
        let p2 = CtPermission::default();
        assert!(p2.printable);
    }

    #[test]
    fn test_ct_permission_read_only_and_xml() {
        let p = CtPermission::new().read_only();
        assert!(!p.editable);
        assert!(!p.annotatable);
        let xml = p.to_xml_string();
        assert!(xml.contains("Editable=\"false\""));
        assert!(xml.contains("Annotatable=\"false\""));
        assert!(xml.contains("Printable=\"true\""));
    }
}
