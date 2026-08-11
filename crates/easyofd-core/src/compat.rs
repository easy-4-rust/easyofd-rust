//! ofdrw Java 类名兼容别名。
//!
//! 将 ofdrw Java 项目中的类名映射到 easyofd-rust 中已有的等价类型。
//! 这些 `pub type` 别名不引入新逻辑，仅用于降低从 Java 迁移时的认知负担。

// ── 页面对象 ──────────────────────────────────────────────────────────────

/// 图层类型枚举。
///
/// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.Type
///
/// 等价于 [`crate::page_obj::LayerType`]。
pub use crate::page_obj::LayerType as Type;

/// 模板页定义。
///
/// 对应 Java: org.ofdrw.core.basicStructure.pageObj.Template
///
/// 等价于 [`crate::page_obj::CT_TemplatePage`]。
pub use crate::page_obj::CT_TemplatePage as Template;

// ── XML 元素 ──────────────────────────────────────────────────────────────

/// OFD XML 元素 trait。
///
/// 对应 Java: org.ofdrw.core.OFDElement
///
/// 等价于 [`crate::xml_element::XmlElement`]。
pub use crate::xml_element::XmlElement as OFDElement;

/// OFD 简单类型元素 trait。
///
/// 对应 Java: org.ofdrw.core.OFDSimpleTypeElement
///
/// 等价于 [`crate::ofd_element::OfdSimpleTypeElement`]。
pub use crate::ofd_element::OfdSimpleTypeElement as OFDSimpleTypeElement;

// ── 图形 ──────────────────────────────────────────────────────────────────

/// 填充规则。
///
/// 对应 Java: org.ofdrw.core.graph.pathObj.Rule
///
/// 等价于 [`crate::graph::FillRule`]。
pub use crate::graph::FillRule as Rule;

// ── 加密 / 签名参数 ──────────────────────────────────────────────────────

/// 加密参数。
///
/// 对应 Java: org.ofdrw.core.crypto.encryt.Parameter
///
/// 等价于 [`crate::crypto::CryptoParameter`]。
pub use crate::crypto::CryptoParameter as Parameter;

/// 签名参数列表。
///
/// 对应 Java: org.ofdrw.core.signatures.sig.Parameters
///
/// 等价于 [`crate::crypto::SigParameters`]。
pub use crate::crypto::SigParameters as Parameters;

// ── 附件 ──────────────────────────────────────────────────────────────────

/// 附件。
///
/// 对应 Java: org.ofdrw.core.attachment.CT_Attachment
///
/// 等价于 [`crate::attachment::CTAttachment`]。
pub use crate::attachment::CTAttachment as CT_Attachment;

// ── 文档信息 ──────────────────────────────────────────────────────────────

/// 文档元数据信息。
///
/// 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_DocInfo
///
/// 等价于 [`crate::doc::ct_doc_info::CtDocInfo`]。
pub use crate::doc::ct_doc_info::CtDocInfo as CT_DocInfo;

// ── 资源 ──────────────────────────────────────────────────────────────────

/// 资源文件容器。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.OFDResource
///
/// 等价于 [`crate::doc::res::Res`]。
pub use crate::doc::res::Res as OFDResource;

// ── 动作 ──────────────────────────────────────────────────────────────────

/// 动作 trait。
///
/// 对应 Java: org.ofdrw.core.action.actionType.OFDAction
///
/// 等价于 [`crate::action::OfdAction`]。
pub use crate::action::OfdAction as OFDAction;

/// 跳转目标 trait。
///
/// 对应 Java: org.ofdrw.core.action.actionType.actionGoto.OFDGotoTarget
///
/// 等价于 [`crate::action::OfdGotoTarget`]。
pub use crate::action::OfdGotoTarget as OFDGotoTarget;

/// 目标位置。
///
/// 对应 Java: org.ofdrw.core.action.actionType.actionGoto.CT_Dest
///
/// 等价于 [`crate::action::CTDest`]。
pub use crate::action::CTDest as CT_Dest;

/// 动作基类。
///
/// 对应 Java: org.ofdrw.core.action.CT_Action
///
/// 等价于 [`crate::action::CTAction`]。
pub use crate::action::CTAction as CT_Action;

// ── 权限 / 视图首选项 / 扩展 ──────────────────────────────────────────────

/// 权限容器。
///
/// 对应 Java: org.ofdrw.core.basicStructure.doc.permission.CT_Permission
///
/// 等价于 [`crate::doc::permission::CtPermission`]。
pub use crate::doc::permission::CtPermission as CT_Permission;

/// 视图首选项。
///
/// 对应 Java: org.ofdrw.core.basicStructure.doc.vpreferences.CT_VPreferences
///
/// 等价于 [`crate::doc::ct_v_preferences::CtVPreferences`]。
pub use crate::doc::ct_v_preferences::CtVPreferences as CT_VPreferences;

/// 单个扩展。
///
/// 对应 Java: org.ofdrw.core.extensions.CT_Extension
///
/// 等价于 [`crate::extensions::CtExtension`]。
pub use crate::extensions::CtExtension as CT_Extension;

// ── 常量 ──────────────────────────────────────────────────────────────────

/// OFD 标准常量。
///
/// 对应 Java: org.ofdrw.core.Const
///
/// 等价于 [`crate::consts`] 模块中的常量。
pub use crate::consts as Const;

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
