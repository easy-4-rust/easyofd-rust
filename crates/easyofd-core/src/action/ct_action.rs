//! 动作基类。
//!
//! 对应 Java: org.ofdrw.core.action.CT_Action

use super::EventType;
use super::OfdAction;

/// 动作基类。
///
/// 所有具体动作类型的基类，包含事件类型和关联的动作列表。
///
/// 对应 Java: org.ofdrw.core.action.CT_Action
#[derive(Debug, Clone)]
pub struct CTAction {
    /// 触发动作的事件类型。
    ///
    /// 对应 Java: CT_Action.eventType (EventType)
    pub event_type: EventType,

    /// 子动作列表。
    ///
    /// 一个事件可以触发多个动作。
    /// 对应 Java: CT_Action 中的子 Action 元素列表
    pub actions: Vec<Box<dyn OfdAction>>,
}

impl CTAction {
    /// 创建一个新的动作基类实例。
    ///
    /// 对应 Java: new CT_Action(EventType)
    #[must_use]
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            actions: Vec::new(),
        }
    }

    /// 添加一个子动作。
    ///
    /// 对应 Java: CT_Action.add(CT_Action)
    pub fn add_action(&mut self, action: Box<dyn OfdAction>) {
        self.actions.push(action);
    }

    /// 序列化为 OFD XML 字符串。
    ///
    /// 输出标准 OFD XML 格式的动作元素。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!("<ofd:CT_Action EventType=\"{}\">", self.event_type);
        for action in &self.actions {
            xml.push_str(&action.to_xml_string());
        }
        xml.push_str("</ofd:CT_Action>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 简单的测试用动作实现。
    #[derive(Debug, Clone)]
    struct TestAction {
        name: String,
    }

    impl OfdAction for TestAction {
        fn to_xml_string(&self) -> String {
            format!("<ofd:TestAction Name=\"{}\"/>", self.name)
        }

        fn clone_box(&self) -> Box<dyn OfdAction> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_ct_action_new() {
        let action = CTAction::new(EventType::PO_DocumentOpen);
        assert_eq!(action.event_type, EventType::PO_DocumentOpen);
        assert!(action.actions.is_empty());
    }

    #[test]
    fn test_ct_action_to_xml_empty() {
        let action = CTAction::new(EventType::PO_ButtonClick);
        let xml = action.to_xml_string();
        assert!(xml.contains("EventType=\"PO_ButtonClick\""));
        assert!(xml.contains("<ofd:CT_Action"));
        assert!(xml.contains("</ofd:CT_Action>"));
    }

    #[test]
    fn test_ct_action_with_children() {
        let mut action = CTAction::new(EventType::PO_DocumentOpen);
        action.add_action(Box::new(TestAction {
            name: "test".to_string(),
        }));
        let xml = action.to_xml_string();
        assert!(xml.contains("Name=\"test\""));
        assert!(xml.contains("<ofd:TestAction"));
    }

    #[test]
    fn test_ct_action_clone() {
        let action = CTAction::new(EventType::PO_PageVisible);
        let action2 = action.clone();
        assert_eq!(action2.event_type, EventType::PO_PageVisible);
    }
}
