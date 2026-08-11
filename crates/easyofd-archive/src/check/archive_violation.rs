//! OFD-A 不合规项描述。
//!
//! 对应 Java: org.ofdrw.archive.check.ArchiveViolation

/// 违规严重程度。
///
/// 对应 Java: org.ofdrw.archive.check.ArchiveViolation.Severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// 错误：不符合 OFD-A 标准的核心约束，必须修复。
    Error,
    /// 警告：不符合标准建议，但可接受。
    Warn,
    /// 信息：提示性信息。
    Info,
}

impl Severity {
    /// 转为字符串表示。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// OFD-A 不合规项描述。
///
/// 检查器对单条规则检查后产生的结果，描述文档中一处不符合
/// GB/T 42133-2022 的位置和原因。
///
/// 对应 Java: org.ofdrw.archive.check.ArchiveViolation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveViolation {
    /// 规则标识，如 "DOC_TYPE"、"IMAGE_FORMAT"。
    rule_name: String,
    /// 严重程度。
    severity: Severity,
    /// 人类可读的问题描述。
    description: String,
    /// 文档中的位置（OFD 容器内绝对路径），可为 None。
    location: Option<String>,
    /// 实际检测到的值，可为 None。
    actual_value: Option<String>,
    /// 标准期望的值，可为 None。
    expected_value: Option<String>,
}

impl ArchiveViolation {
    /// 创建不合规项描述。
    ///
    /// # Arguments
    ///
    /// * `rule_name` - 规则标识（如 "DOC_TYPE"）
    /// * `severity` - 严重程度
    /// * `description` - 问题描述
    /// * `location` - 文档内位置（OFD 容器内路径），可为 None
    /// * `actual_value` - 实际值，可为 None
    /// * `expected_value` - 期望值，可为 None
    pub fn new(
        rule_name: impl Into<String>,
        severity: Severity,
        description: impl Into<String>,
        location: Option<impl Into<String>>,
        actual_value: Option<impl Into<String>>,
        expected_value: Option<impl Into<String>>,
    ) -> Self {
        Self {
            rule_name: rule_name.into(),
            severity,
            description: description.into(),
            location: location.map(Into::into),
            actual_value: actual_value.map(Into::into),
            expected_value: expected_value.map(Into::into),
        }
    }

    /// 创建错误级别的违规项。
    pub fn error(rule_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(
            rule_name,
            Severity::Error,
            description,
            None::<String>,
            None::<String>,
            None::<String>,
        )
    }

    /// 创建警告级别的违规项。
    pub fn warn(rule_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(
            rule_name,
            Severity::Warn,
            description,
            None::<String>,
            None::<String>,
            None::<String>,
        )
    }

    /// 创建信息级别的违规项。
    pub fn info(rule_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(
            rule_name,
            Severity::Info,
            description,
            None::<String>,
            None::<String>,
            None::<String>,
        )
    }

    /// 获取规则标识。
    #[must_use]
    pub fn rule_name(&self) -> &str {
        &self.rule_name
    }

    /// 获取严重程度。
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// 获取问题描述。
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 获取文档内位置。
    #[must_use]
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// 获取实际值。
    #[must_use]
    pub fn actual_value(&self) -> Option<&str> {
        self.actual_value.as_deref()
    }

    /// 获取期望值。
    #[must_use]
    pub fn expected_value(&self) -> Option<&str> {
        self.expected_value.as_deref()
    }

    /// 是否为错误级别。
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl std::fmt::Display for ArchiveViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.severity, self.rule_name, self.description
        )?;
        if let Some(ref loc) = self.location {
            write!(f, " (位置: {loc})")?;
        }
        if let Some(ref actual) = self.actual_value {
            write!(f, " 实际: {actual}")?;
        }
        if let Some(ref expected) = self.expected_value {
            write!(f, " 期望: {expected}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Error.to_string(), "ERROR");
        assert_eq!(Severity::Warn.to_string(), "WARN");
        assert_eq!(Severity::Info.to_string(), "INFO");
    }

    #[test]
    fn severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "ERROR");
        assert_eq!(Severity::Warn.as_str(), "WARN");
        assert_eq!(Severity::Info.as_str(), "INFO");
    }

    #[test]
    fn violation_new() {
        let v = ArchiveViolation::new(
            "DOC_TYPE",
            Severity::Error,
            "DocType 不是 OFD-A",
            Some("OFD.xml"),
            Some("OFD"),
            Some("OFD-A"),
        );
        assert_eq!(v.rule_name(), "DOC_TYPE");
        assert_eq!(v.severity(), Severity::Error);
        assert_eq!(v.description(), "DocType 不是 OFD-A");
        assert_eq!(v.location(), Some("OFD.xml"));
        assert_eq!(v.actual_value(), Some("OFD"));
        assert_eq!(v.expected_value(), Some("OFD-A"));
        assert!(v.is_error());
    }

    #[test]
    fn violation_error_shortcut() {
        let v = ArchiveViolation::error("TEST", "测试错误");
        assert_eq!(v.severity(), Severity::Error);
        assert!(v.location().is_none());
    }

    #[test]
    fn violation_warn_shortcut() {
        let v = ArchiveViolation::warn("TEST", "测试警告");
        assert_eq!(v.severity(), Severity::Warn);
        assert!(!v.is_error());
    }

    #[test]
    fn violation_info_shortcut() {
        let v = ArchiveViolation::info("TEST", "测试信息");
        assert_eq!(v.severity(), Severity::Info);
    }

    #[test]
    fn violation_display() {
        let v = ArchiveViolation::new(
            "DOC_TYPE",
            Severity::Error,
            "DocType 不匹配",
            Some("OFD.xml"),
            Some("OFD"),
            Some("OFD-A"),
        );
        let s = v.to_string();
        assert!(s.contains("[ERROR]"));
        assert!(s.contains("DOC_TYPE"));
        assert!(s.contains("DocType 不匹配"));
        assert!(s.contains("OFD.xml"));
        assert!(s.contains("OFD"));
    }

    #[test]
    fn violation_display_minimal() {
        let v = ArchiveViolation::error("X", "y");
        let s = v.to_string();
        assert_eq!(s, "[ERROR] X: y");
    }

    #[test]
    fn violation_clone_eq() {
        let v = ArchiveViolation::error("R", "d");
        let v2 = v.clone();
        assert_eq!(v, v2);
    }

    #[test]
    fn severity_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Severity::Error);
        set.insert(Severity::Warn);
        set.insert(Severity::Info);
        assert_eq!(set.len(), 3);
    }
}
