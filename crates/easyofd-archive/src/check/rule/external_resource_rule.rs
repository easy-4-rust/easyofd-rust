//! 规则 3：资源必须自包含，禁止外部引用。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ExternalResourceRule
//! GB/T 42133-2022 6.1

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 规则 3：资源必须自包含，禁止外部引用。
///
/// 检查所有 XML 文件中是否包含 http/https URL 引用。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ExternalResourceRule
#[derive(Debug, Clone, Copy)]
pub struct ExternalResourceRule;

impl ComplianceRule for ExternalResourceRule {
    fn name(&self) -> &'static str {
        "EXTERNAL_RESOURCE"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("http://") || content.contains("https://") {
                    // 排除命名空间声明
                    let has_external = content.lines().any(|line| {
                        let trimmed = line.trim();
                        (trimmed.contains("http://") || trimmed.contains("https://"))
                            && !trimmed.contains("xmlns")
                            && !trimmed.contains("ofdspec.org")
                            && !trimmed.contains("w3.org")
                    });
                    if has_external {
                        return crate::rules::RuleResult {
                            passed: false,
                            message: format!("文件 {name} 包含外部资源引用"),
                        };
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "未发现外部资源引用".into(),
        }
    }
}

impl ExternalResourceRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut violations = Vec::new();
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                for line in content.lines() {
                    let trimmed = line.trim();
                    if (trimmed.contains("http://") || trimmed.contains("https://"))
                        && !trimmed.contains("xmlns")
                        && !trimmed.contains("ofdspec.org")
                        && !trimmed.contains("w3.org")
                    {
                        violations.push(ArchiveViolation::new(
                            self.name(),
                            Severity::Error,
                            "引用外部 URL 资源",
                            Some(name.as_str()),
                            None::<String>,
                            Some("包内路径"),
                        ));
                        break;
                    }
                }
            }
        }
        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_resource_rule_name() {
        assert_eq!(ExternalResourceRule.name(), "EXTERNAL_RESOURCE");
    }

    #[test]
    fn external_resource_rule_passes_without_external() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br#"<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
                <ofd:Pages><ofd:Page ID="1" BaseLoc="Pages/Page_0.xml"/></ofd:Pages>
            </ofd:Document>"#
                .to_vec(),
        )];
        assert!(ExternalResourceRule.check(&entries).passed);
    }

    #[test]
    fn external_resource_rule_fails_with_http() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:TextCode>http://evil.com/steal</ofd:TextCode></ofd:Document>"
                .to_vec(),
        )];
        assert!(!ExternalResourceRule.check(&entries).passed);
    }
}
