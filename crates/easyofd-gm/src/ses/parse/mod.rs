//! SES 电子签章版本解析模块。
//!
//! 对应 Java: `org.ofdrw.gm.ses.parse`
//!
//! 提供版本枚举 [`SESVersion`]、版本持有器 [`SESVersionHolder`]
//! 与版本探测 [`VersionParser`]，用于从原始 DER 字节自动识别
//! SES V1 / V4 / V5 结构。

mod ses_version;
mod ses_version_holder;
mod version_parser;

pub use ses_version::SESVersion;
pub use ses_version_holder::SESVersionHolder;
pub use version_parser::VersionParser;
