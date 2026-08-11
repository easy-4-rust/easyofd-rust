//! 签章 ID 解析器。
//!
//! 对应 Java: `org.ofdrw.sign.SignIdParser`、`org.ofdrw.sign.NumberFormatAtomicSignID`、
//! `org.ofdrw.sign.StandFormatAtomicSignID`

use std::sync::atomic::{AtomicU32, Ordering};

/// 签章 ID 解析器。
///
/// 对应 Java: `org.ofdrw.sign.SignIdParser`
///
/// 从签章 ID 字符串中解析数字部分。
pub struct SignIdParser;

impl SignIdParser {
    /// 从 `sNNN` 格式解析数字部分。
    ///
    /// # 示例
    ///
    /// ```
    /// use easyofd_signature::sign_id_parser::SignIdParser;
    ///
    /// assert_eq!(SignIdParser::parse_standard("s001"), Some(1));
    /// assert_eq!(SignIdParser::parse_standard("s010"), Some(10));
    /// assert_eq!(SignIdParser::parse_standard("abc"), None);
    /// ```
    #[must_use]
    pub fn parse_standard(id: &str) -> Option<u32> {
        let stripped = id.strip_prefix('s').unwrap_or(id);
        stripped.parse::<u32>().ok()
    }

    /// 从纯数字格式解析。
    #[must_use]
    pub fn parse_number(id: &str) -> Option<u32> {
        id.parse::<u32>().ok()
    }
}

/// 数字格式原子签章 ID 提供者。
///
/// 对应 Java: `org.ofdrw.sign.NumberFormatAtomicSignID`
///
/// 使用纯递增数字作为签章 ID。
#[derive(Debug)]
pub struct NumberFormatAtomicSignId {
    counter: AtomicU32,
}

impl NumberFormatAtomicSignId {
    /// 创建新的数字格式 ID 提供者。
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter: AtomicU32::new(1),
        }
    }

    /// 从指定起始值创建。
    #[must_use]
    pub fn with_start(start: u32) -> Self {
        Self {
            counter: AtomicU32::new(start),
        }
    }

    /// 获取下一个 ID（原子自增）。
    pub fn next_id(&self) -> String {
        let val = self.counter.fetch_add(1, Ordering::SeqCst);
        val.to_string()
    }

    /// 设置当前最大 ID。
    pub fn set_current_max(&self, max_id: u32) {
        self.counter.store(max_id + 1, Ordering::SeqCst);
    }
}

impl Default for NumberFormatAtomicSignId {
    fn default() -> Self {
        Self::new()
    }
}

/// 标准格式原子签章 ID 提供者。
///
/// 对应 Java: `org.ofdrw.sign.StandFormatAtomicSignID`
///
/// 使用 `sNNN` 格式（自动补零到 3 位）作为签章 ID。
#[derive(Debug)]
pub struct StandFormatAtomicSignId {
    counter: AtomicU32,
    width: usize,
}

impl StandFormatAtomicSignId {
    /// 创建新的标准格式 ID 提供者。
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter: AtomicU32::new(1),
            width: 3,
        }
    }

    /// 设置补零宽度。
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// 获取下一个 ID（原子自增）。
    pub fn next_id(&self) -> String {
        let val = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("s{:0>width$}", val, width = self.width)
    }

    /// 设置当前最大 ID。
    pub fn set_current_max(&self, max_id: u32) {
        self.counter.store(max_id + 1, Ordering::SeqCst);
    }
}

impl Default for StandFormatAtomicSignId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_id_parser_standard() {
        assert_eq!(SignIdParser::parse_standard("s001"), Some(1));
        assert_eq!(SignIdParser::parse_standard("s010"), Some(10));
        assert_eq!(SignIdParser::parse_standard("s100"), Some(100));
        assert_eq!(SignIdParser::parse_standard("abc"), None);
    }

    #[test]
    fn sign_id_parser_number() {
        assert_eq!(SignIdParser::parse_number("1"), Some(1));
        assert_eq!(SignIdParser::parse_number("42"), Some(42));
        assert_eq!(SignIdParser::parse_number("abc"), None);
    }

    #[test]
    fn number_format_atomic_id() {
        let provider = NumberFormatAtomicSignId::new();
        assert_eq!(provider.next_id(), "1");
        assert_eq!(provider.next_id(), "2");
        assert_eq!(provider.next_id(), "3");
    }

    #[test]
    fn number_format_atomic_id_set_max() {
        let provider = NumberFormatAtomicSignId::new();
        provider.set_current_max(10);
        assert_eq!(provider.next_id(), "11");
    }

    #[test]
    fn number_format_atomic_id_with_start() {
        let provider = NumberFormatAtomicSignId::with_start(100);
        assert_eq!(provider.next_id(), "100");
        assert_eq!(provider.next_id(), "101");
    }

    #[test]
    fn stand_format_atomic_id() {
        let provider = StandFormatAtomicSignId::new();
        assert_eq!(provider.next_id(), "s001");
        assert_eq!(provider.next_id(), "s002");
        assert_eq!(provider.next_id(), "s003");
    }

    #[test]
    fn stand_format_atomic_id_width() {
        let provider = StandFormatAtomicSignId::new().with_width(5);
        assert_eq!(provider.next_id(), "s00001");
    }

    #[test]
    fn stand_format_atomic_id_set_max() {
        let provider = StandFormatAtomicSignId::new();
        provider.set_current_max(99);
        assert_eq!(provider.next_id(), "s100");
    }
}
