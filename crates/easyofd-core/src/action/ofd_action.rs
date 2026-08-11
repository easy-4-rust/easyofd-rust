//! 动作通用 trait。
//!
//! 对应 Java: org.ofdrw.core.action.IAction

/// 动作通用接口。
///
/// 所有动作类型（URI、Goto、GotoA、Sound、Movie 等）都实现此 trait。
///
/// 对应 Java: org.ofdrw.core.action.IAction
pub trait OfdAction {
    /// 序列化为 OFD XML 字符串。
    ///
    /// 输出标准 OFD XML 格式的动作元素。
    fn to_xml_string(&self) -> String;

    /// 克隆为 Boxed trait object。
    ///
    /// 用于支持 `Box<dyn OfdAction>` 的克隆操作。
    fn clone_box(&self) -> Box<dyn OfdAction>;
}

/// 为 `Box<dyn OfdAction>` 实现 `Clone`。
impl Clone for Box<dyn OfdAction> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// 为 `Box<dyn OfdAction>` 实现 `Debug`。
impl std::fmt::Debug for Box<dyn OfdAction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OfdAction(\"{}\")", self.to_xml_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用的简单动作实现。
    #[derive(Debug, Clone)]
    struct DummyAction {
        tag_name: String,
    }

    impl OfdAction for DummyAction {
        fn to_xml_string(&self) -> String {
            format!("<{}/>", self.tag_name)
        }

        fn clone_box(&self) -> Box<dyn OfdAction> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_ofd_action_trait_to_xml() {
        let action = DummyAction {
            tag_name: "TestAction".to_string(),
        };
        assert_eq!(action.to_xml_string(), "<TestAction/>");
    }

    #[test]
    fn test_ofd_action_trait_object() {
        let action: Box<dyn OfdAction> = Box::new(DummyAction {
            tag_name: "Boxed".to_string(),
        });
        assert_eq!(action.to_xml_string(), "<Boxed/>");
    }

    #[test]
    fn test_ofd_action_clone_box() {
        let action: Box<dyn OfdAction> = Box::new(DummyAction {
            tag_name: "Clone".to_string(),
        });
        let cloned = action.clone_box();
        assert_eq!(cloned.to_xml_string(), "<Clone/>");
    }

    #[test]
    fn test_ofd_action_debug_box() {
        let action: Box<dyn OfdAction> = Box::new(DummyAction {
            tag_name: "Dbg".to_string(),
        });
        let dbg = format!("{action:?}");
        assert!(dbg.contains("OfdAction"));
        assert!(dbg.contains("<Dbg/>"));
    }
}
