//! 事件类型枚举。
//!
//! 对应 Java: org.ofdrw.core.action.EventType

/// 事件类型。
///
/// 定义触发动作的事件类型，对应 GB/T 33190 第 15 章。
///
/// 对应 Java: org.ofdrw.core.action.EventType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EventType {
    /// 文档打开事件。
    ///
    /// 当文档被打开时触发关联的动作。
    PO_DocumentOpen,

    /// 文档关闭事件。
    ///
    /// 当文档被关闭时触发关联的动作。
    PO_DocumentClose,

    /// 按钮点击事件。
    ///
    /// 当用户点击按钮时触发关联的动作。
    PO_ButtonClick,

    /// 页面可见事件。
    ///
    /// 当页面变为可见时触发关联的动作。
    PO_PageVisible,

    /// 页面不可见事件。
    ///
    /// 当页面变为不可见时触发关联的动作。
    PO_PageInvisible,
}

impl EventType {
    /// 返回事件类型的 OFD XML 属性值。
    ///
    /// 对应 Java: EventType 的 toString()/value
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PO_DocumentOpen => "PO_DocumentOpen",
            Self::PO_DocumentClose => "PO_DocumentClose",
            Self::PO_ButtonClick => "PO_ButtonClick",
            Self::PO_PageVisible => "PO_PageVisible",
            Self::PO_PageInvisible => "PO_PageInvisible",
        }
    }

    /// 从字符串解析事件类型。
    ///
    /// # Errors
    ///
    /// 如果字符串不匹配任何已知事件类型，返回错误。
    pub fn from_str_value(s: &str) -> Result<Self, String> {
        match s {
            "PO_DocumentOpen" => Ok(Self::PO_DocumentOpen),
            "PO_DocumentClose" => Ok(Self::PO_DocumentClose),
            "PO_ButtonClick" => Ok(Self::PO_ButtonClick),
            "PO_PageVisible" => Ok(Self::PO_PageVisible),
            "PO_PageInvisible" => Ok(Self::PO_PageInvisible),
            _ => Err(format!("unknown EventType: {s}")),
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::PO_DocumentOpen.as_str(), "PO_DocumentOpen");
        assert_eq!(EventType::PO_DocumentClose.as_str(), "PO_DocumentClose");
        assert_eq!(EventType::PO_ButtonClick.as_str(), "PO_ButtonClick");
        assert_eq!(EventType::PO_PageVisible.as_str(), "PO_PageVisible");
        assert_eq!(EventType::PO_PageInvisible.as_str(), "PO_PageInvisible");
    }

    #[test]
    fn test_event_type_from_str_roundtrip() {
        let variants = [
            EventType::PO_DocumentOpen,
            EventType::PO_DocumentClose,
            EventType::PO_ButtonClick,
            EventType::PO_PageVisible,
            EventType::PO_PageInvisible,
        ];
        for v in &variants {
            let s = v.as_str();
            let parsed = EventType::from_str_value(s).unwrap();
            assert_eq!(*v, parsed);
        }
    }

    #[test]
    fn test_event_type_from_str_invalid() {
        assert!(EventType::from_str_value("INVALID").is_err());
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", EventType::PO_DocumentOpen), "PO_DocumentOpen");
    }

    #[test]
    fn test_event_type_clone_copy() {
        let a = EventType::PO_ButtonClick;
        let b = a;
        assert_eq!(a, b);
        let c = a;
        assert_eq!(a, c);
    }
}
