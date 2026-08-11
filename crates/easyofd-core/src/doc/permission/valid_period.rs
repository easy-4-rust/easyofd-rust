//! 有效期。

/// 对应 Java: org.ofdrw.core.basicStructure.ValidPeriod
///
/// 文档有效期控制，指定文档的有效起止时间。
#[derive(Debug, Clone)]
pub struct ValidPeriod {
    /// 有效开始日期（如 "2024-01-01"）。
    pub start_date: String,
    /// 有效结束日期（如 "2024-12-31"）。
    pub end_date: String,
}

impl ValidPeriod {
    /// 创建有效期。
    #[must_use]
    pub fn new(start_date: impl Into<String>, end_date: impl Into<String>) -> Self {
        Self {
            start_date: start_date.into(),
            end_date: end_date.into(),
        }
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        format!(
            "<ValidPeriod StartDate=\"{}\" EndDate=\"{}\"/>",
            self.start_date, self.end_date
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_period_new() {
        let vp = ValidPeriod::new("2024-01-01", "2024-12-31");
        assert_eq!(vp.start_date, "2024-01-01");
        assert_eq!(vp.end_date, "2024-12-31");
    }

    #[test]
    fn test_valid_period_xml() {
        let vp = ValidPeriod::new("2024-01-01", "2024-12-31");
        let xml = vp.to_xml_string();
        assert!(xml.contains("StartDate=\"2024-01-01\""));
        assert!(xml.contains("EndDate=\"2024-12-31\""));
        assert!(xml.contains("/>"));
    }
}
