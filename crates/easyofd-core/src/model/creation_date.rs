//! OFD 创建/修改日期。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_DocInfo (CreationDate/ModDate)

use chrono::NaiveDateTime;

/// 日期时间值（ofd:CreationDate / ofd:ModDate）。
///
/// 对应 Java: ofdrw CreationDate/ModDate 字段。
/// 支持 ISO 格式 "2024-05-31T00:00:00" 和日期格式 "2024-05-31"。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreationDate {
    /// 解析后的日期时间值。
    pub value: NaiveDateTime,
}

impl CreationDate {
    /// 从 `NaiveDateTime` 创建。
    pub fn new(value: NaiveDateTime) -> Self {
        Self { value }
    }

    /// 从 ISO 字符串解析。
    ///
    /// 支持 "2024-05-31T00:00:00" 和 "2024-05-31" 格式。
    pub fn parse(s: &str) -> Option<Self> {
        let value = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).expect("00:00:00 是有效时间"))
            })
            .ok()?;
        Some(Self { value })
    }

    /// 转换为 ISO 字符串。
    #[must_use]
    pub fn to_iso(&self) -> String {
        self.value.format("%Y-%m-%dT%H:%M:%S").to_string()
    }
}
