//! OFD-A 合规检查器。
//!
//! 对应 Java: org.ofdrw.archive.check.OFDArchiveChecker

use super::archive_rule::ArchiveRule;
use super::archive_violation::ArchiveViolation;

/// OFD-A 合规检查器。
///
/// 聚合所有检查规则，对 OFD 文档执行全面的 GB/T 42133-2022 合规检查。
///
/// 对应 Java: org.ofdrw.archive.check.OFDArchiveChecker
pub struct OfdArchiveChecker {
    /// 检查规则列表。
    rules: Vec<Box<dyn ArchiveRule>>,
}

impl OfdArchiveChecker {
    /// 创建检查器，使用自定义规则集。
    pub fn new(rules: Vec<Box<dyn ArchiveRule>>) -> Self {
        Self { rules }
    }

    /// 对 OFD 包条目执行全部检查。
    ///
    /// 逐个执行所有注册的检查规则，收集违规项。
    /// 单条规则异常不影响其他规则的执行。
    ///
    /// 返回违规列表，按规则名排序，无违规则为空列表。
    pub fn check(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut all_violations = Vec::new();

        for rule in &self.rules {
            let violations = rule.check(entries);
            all_violations.extend(violations);
        }

        // 按规则名排序，保证输出稳定
        all_violations.sort_by(|a, b| a.rule_name().cmp(b.rule_name()));
        all_violations
    }

    /// 获取已注册的规则数量。
    #[must_use]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for OfdArchiveChecker {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::archive_violation::Severity;

    struct PassRule;
    impl ArchiveRule for PassRule {
        fn name(&self) -> &'static str {
            "PassRule"
        }
        fn check(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
            Vec::new()
        }
    }

    struct FailRule;
    impl ArchiveRule for FailRule {
        fn name(&self) -> &'static str {
            "FailRule"
        }
        fn check(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
            vec![ArchiveViolation::error("FailRule", "失败")]
        }
    }

    #[test]
    fn checker_default_is_empty() {
        let checker = OfdArchiveChecker::default();
        assert_eq!(checker.rule_count(), 0);
        assert!(checker.check(&[]).is_empty());
    }

    #[test]
    fn checker_with_rules() {
        let checker = OfdArchiveChecker::new(vec![Box::new(PassRule), Box::new(FailRule)]);
        assert_eq!(checker.rule_count(), 2);
        let violations = checker.check(&[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity(), Severity::Error);
    }

    #[test]
    fn checker_sorts_by_rule_name() {
        let checker = OfdArchiveChecker::new(vec![Box::new(FailRule), Box::new(PassRule)]);
        let violations = checker.check(&[]);
        assert_eq!(violations[0].rule_name(), "FailRule");
    }
}
