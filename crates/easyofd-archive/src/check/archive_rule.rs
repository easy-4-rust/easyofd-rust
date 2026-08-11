//! OFD-A 检查规则 trait。
//!
//! 对应 Java: org.ofdrw.archive.check.ArchiveRule

use super::archive_violation::ArchiveViolation;

/// OFD-A 检查规则。
///
/// 每条规则验证 OFD 文档的某一个约束是否满足 GB/T 42133-2022 标准。
/// 规则实现应为无状态，允许多次调用。
///
/// 对应 Java: org.ofdrw.archive.check.ArchiveRule
pub trait ArchiveRule: Send + Sync {
    /// 规则名称。
    fn name(&self) -> &'static str;

    /// 对 OFD 包条目执行检查。
    ///
    /// `entries` 是 OFD 包内所有文件的 `(路径, 内容)` 列表。
    ///
    /// 返回发现的违规项列表（非 null），文档合规时返回空列表。
    fn check(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::archive_violation::Severity;

    struct MockRule;

    impl ArchiveRule for MockRule {
        fn name(&self) -> &'static str {
            "MockRule"
        }

        fn check(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
            vec![ArchiveViolation::error("MockRule", "测试违规")]
        }
    }

    #[test]
    fn mock_rule_name() {
        let rule = MockRule;
        assert_eq!(rule.name(), "MockRule");
    }

    #[test]
    fn mock_rule_check() {
        let rule = MockRule;
        let violations = rule.check(&[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity(), Severity::Error);
    }
}
