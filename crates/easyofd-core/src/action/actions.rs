//! 动作容器。
//!
//! 对应 Java: org.ofdrw.core.action.Actions

use super::OfdAction;

/// 动作容器。
///
/// 包含一组动作的有序序列，对应 GB/T 33190 第 15 章的 Actions 元素。
///
/// 对应 Java: org.ofdrw.core.action.Actions
#[derive(Debug, Clone)]
pub struct Actions {
    /// 动作列表。
    ///
    /// 对应 Java: Actions 中的子 Action 元素列表
    pub actions: Vec<Box<dyn OfdAction>>,
}

impl Actions {
    /// 创建一个新的空动作容器。
    ///
    /// 对应 Java: new Actions()
    #[must_use]
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// 添加一个动作。
    ///
    /// 对应 Java: Actions.add(OfdAction)
    pub fn push(&mut self, action: Box<dyn OfdAction>) {
        self.actions.push(action);
    }

    /// 返回动作数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// 判断是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// 序列化为 OFD XML 字符串。
    ///
    /// 输出标准 OFD XML 格式的动作容器元素。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = String::from("<ofd:Actions>");
        for action in &self.actions {
            xml.push_str(&action.to_xml_string());
        }
        xml.push_str("</ofd:Actions>");
        xml
    }
}

impl Default for Actions {
    fn default() -> Self {
        Self::new()
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
            format!("<ofd:Test Name=\"{}\"/>", self.name)
        }

        fn clone_box(&self) -> Box<dyn OfdAction> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_actions_new() {
        let actions = Actions::new();
        assert!(actions.is_empty());
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_actions_push() {
        let mut actions = Actions::new();
        actions.push(Box::new(TestAction {
            name: "a1".to_string(),
        }));
        assert_eq!(actions.len(), 1);
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_actions_to_xml_empty() {
        let actions = Actions::new();
        let xml = actions.to_xml_string();
        assert_eq!(xml, "<ofd:Actions></ofd:Actions>");
    }

    #[test]
    fn test_actions_to_xml_with_children() {
        let mut actions = Actions::new();
        actions.push(Box::new(TestAction {
            name: "action1".to_string(),
        }));
        actions.push(Box::new(TestAction {
            name: "action2".to_string(),
        }));
        let xml = actions.to_xml_string();
        assert!(xml.contains("<ofd:Actions>"));
        assert!(xml.contains("Name=\"action1\""));
        assert!(xml.contains("Name=\"action2\""));
        assert!(xml.contains("</ofd:Actions>"));
    }

    #[test]
    fn test_actions_default() {
        let actions = Actions::default();
        assert!(actions.is_empty());
    }

    #[test]
    fn test_actions_clone() {
        let mut actions = Actions::new();
        actions.push(Box::new(TestAction {
            name: "x".to_string(),
        }));
        let actions2 = actions.clone();
        assert_eq!(actions2.len(), 1);
    }
}
