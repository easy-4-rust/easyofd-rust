//! ofdrw Java 类名兼容别名。
//!
//! 将 ofdrw Java 项目中的类名映射到 easyofd-rust 中已有的等价类型。
//! 这些 `pub use` 别名不引入新逻辑，仅用于降低从 Java 迁移时的认知负担。
//!
//! 每个别名单独一个 `.rs` 文件，对应一个 Java 类。

// ── 页面对象 ──────────────────────────────────────────────────────────────
mod layer_type;
mod template_page;

// ── XML 元素 ──────────────────────────────────────────────────────────────
mod ofd_element;
mod ofd_simple_type_element;

// ── 图形 ──────────────────────────────────────────────────────────────────
mod fill_rule;

// ── 加密 / 签名参数 ──────────────────────────────────────────────────────
mod crypto_parameter;
mod sig_parameters;

// ── 附件 ──────────────────────────────────────────────────────────────────
mod ct_attachment;

// ── 文档信息 ──────────────────────────────────────────────────────────────
mod ct_doc_info;

// ── 资源 ──────────────────────────────────────────────────────────────────
mod ofd_resource;

// ── 动作 ──────────────────────────────────────────────────────────────────
mod ct_action;
mod ct_dest;
mod ofd_action;
mod ofd_goto_target;

// ── 权限 / 视图首选项 / 扩展 ──────────────────────────────────────────────
mod ct_extension;
mod ct_permission;
mod ct_v_preferences;

// ── 常量 ──────────────────────────────────────────────────────────────────
mod ofd_const;

// Re-export all aliases at `compat` module level.
pub use crypto_parameter::Parameter;
pub use ct_action::CT_Action;
pub use ct_attachment::CT_Attachment;
pub use ct_dest::CT_Dest;
pub use ct_doc_info::CT_DocInfo;
pub use ct_extension::CT_Extension;
pub use ct_permission::CT_Permission;
pub use ct_v_preferences::CT_VPreferences;
pub use fill_rule::Rule;
pub use layer_type::Type;
pub use ofd_action::OFDAction;
pub use ofd_const::Const;
pub use ofd_element::OFDElement;
pub use ofd_goto_target::OFDGotoTarget;
pub use ofd_resource::OFDResource;
pub use ofd_simple_type_element::OFDSimpleTypeElement;
pub use sig_parameters::Parameters;
pub use template_page::Template;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_alias_is_layer_type() {
        let t = Type::Body;
        assert_eq!(t.as_str(), "Body");
    }

    #[test]
    fn template_alias_works() {
        let tpl = Template::new(1).name("bg");
        assert_eq!(tpl.get_name(), Some("bg"));
    }

    #[test]
    fn rule_alias_is_fill_rule() {
        let r = Rule::EvenOdd;
        assert_eq!(r.as_str(), "EvenOdd");
    }

    #[test]
    fn parameter_alias_works() {
        let p = Parameter::new("key", "val");
        assert_eq!(p.name, "key");
    }

    #[test]
    fn parameters_alias_works() {
        let mut params = Parameters::new();
        params.add(crate::crypto::SigParameter::new("a", "1"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn ct_attachment_alias_works() {
        let a = CT_Attachment::new("1", "test.txt");
        assert_eq!(a.name, "test.txt");
    }

    #[test]
    fn ct_doc_info_alias_works() {
        let info = CT_DocInfo::new("doc1");
        assert_eq!(info.doc_id, "doc1");
    }

    #[test]
    fn ofd_resource_alias_works() {
        let r = OFDResource::new();
        assert_eq!(r.resource_count(), 0);
    }

    #[test]
    fn ct_dest_alias_works() {
        let d = CT_Dest::new(5);
        assert_eq!(d.page, 5);
    }

    #[test]
    fn ct_action_alias_works() {
        let a = CT_Action::new(crate::action::EventType::PO_DocumentOpen);
        assert_eq!(a.event_type, crate::action::EventType::PO_DocumentOpen);
    }

    #[test]
    fn ct_permission_alias_works() {
        let p = CT_Permission::new();
        assert!(p.printable);
    }

    #[test]
    fn ct_v_preferences_alias_works() {
        let vp = CT_VPreferences::new();
        assert_eq!(vp.page_mode, crate::doc::ct_v_preferences::PageMode::None);
    }

    #[test]
    fn ct_extension_alias_works() {
        let ext = CT_Extension::new("myext", "1.0");
        assert_eq!(ext.name, "myext");
    }

    #[test]
    fn const_module_alias_works() {
        assert_eq!(Const::OFD_NAMESPACE, "http://www.ofdspec.org/2016");
    }

    #[test]
    fn ofd_element_alias_is_xml_element() {
        // OFDElement 是 XmlElement 的别名，验证可用作 trait bound
        fn needs_ofd_element<T: OFDElement>(_t: &T) {}
        let path = crate::graph::CT_Path::new(1, "0 0 10 10");
        needs_ofd_element(&path);
    }
}
