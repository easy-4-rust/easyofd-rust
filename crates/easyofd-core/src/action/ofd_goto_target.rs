//! 跳转目标 trait。
//!
//! 对应 Java: org.ofdrw.core.action.IGotoTarget

/// 跳转目标接口。
///
/// 定义页面跳转目标的通用接口，用于 `Goto` 和 `GotoA` 等跳转动作。
///
/// 对应 Java: org.ofdrw.core.action.IGotoTarget
pub trait OfdGotoTarget {
    /// 序列化为 OFD XML 字符串。
    ///
    /// 输出标准 OFD XML 格式的目标元素。
    fn to_xml_string(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用的简单目标实现。
    struct DummyTarget {
        page: u32,
    }

    impl OfdGotoTarget for DummyTarget {
        fn to_xml_string(&self) -> String {
            format!("<Target PageID=\"{}\"/>", self.page)
        }
    }

    #[test]
    fn test_ofd_goto_target_trait_to_xml() {
        let target = DummyTarget { page: 5 };
        assert_eq!(target.to_xml_string(), "<Target PageID=\"5\"/>");
    }

    #[test]
    fn test_ofd_goto_target_trait_object() {
        let target: Box<dyn OfdGotoTarget> = Box::new(DummyTarget { page: 1 });
        assert_eq!(target.to_xml_string(), "<Target PageID=\"1\"/>");
    }
}
