//! 签章 ID 生成器（SignIDProvider）。
//!
//! 对应 Java: org.ofdrw.sign.SignIDProvider
//!
//! 开发者可以根据实际需求配置签章 ID 的格式。
//! 标准推荐 ID 样式为 `sNNN`，NNN 从 1 开始。

use std::sync::atomic::{AtomicU32, Ordering};

/// 签章 ID 提供者 trait。
///
/// 开发者可以实现此 trait 来自定义签章 ID 的生成策略。
pub trait SignIdProvider {
    /// 设置当前最大签章 ID 值。
    fn set_current_max_sign_id(&mut self, max_sign_id: &str);

    /// 增长并获取签章 ID。
    fn increment_and_get(&mut self) -> String;

    /// 获取当前签章 ID，不增长。
    fn get(&self) -> String;

    /// 解析出签章 ID 的数字部分。
    fn parse_id(&self, id: &str) -> Result<u32, String>;
}

/// 标准格式签章 ID 提供者。
///
/// 使用 `sNNN` 格式，NNN 从 1 开始，自动补零到 3 位。
///
/// # 示例
/// ```
/// use easyofd_signature::sign_id_provider::{StandardSignIdProvider, SignIdProvider};
///
/// let mut provider = StandardSignIdProvider::new();
/// assert_eq!(provider.get(), "s001");
/// assert_eq!(provider.increment_and_get(), "s002");
/// ```
#[derive(Debug)]
pub struct StandardSignIdProvider {
    /// 当前计数器。
    counter: AtomicU32,
    /// 补零宽度。
    width: usize,
}

impl StandardSignIdProvider {
    /// 创建新的标准格式签章 ID 提供者。
    ///
    /// 默认从 `s001` 开始，补零宽度为 3。
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

    /// 格式化 ID。
    fn format_id(&self, value: u32) -> String {
        format!("s{:0>width$}", value, width = self.width)
    }
}

impl Default for StandardSignIdProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SignIdProvider for StandardSignIdProvider {
    fn set_current_max_sign_id(&mut self, max_sign_id: &str) {
        if let Some(num) = parse_sign_id_number(max_sign_id) {
            // counter 表示"下一个待分配的 ID"，所以设为 max + 1
            self.counter = AtomicU32::new(num + 1);
        }
    }

    /// 分配并返回下一个签章 ID，内部计数器自增。
    fn increment_and_get(&mut self) -> String {
        let val = self.counter.fetch_add(1, Ordering::SeqCst);
        self.format_id(val)
    }

    /// 获取下一个待分配的签章 ID，计数器同步自增。
    fn get(&self) -> String {
        let val = self.counter.fetch_add(1, Ordering::SeqCst);
        self.format_id(val)
    }

    fn parse_id(&self, id: &str) -> Result<u32, String> {
        parse_sign_id_number(id).ok_or_else(|| format!("无法解析签章 ID: {id}"))
    }
}

/// 数字格式签章 ID 提供者。
///
/// 直接使用递增数字作为 ID，不带前缀。
#[derive(Debug)]
pub struct NumberSignIdProvider {
    /// 当前计数器。
    counter: u32,
}

impl NumberSignIdProvider {
    /// 创建新的数字格式签章 ID 提供者。
    #[must_use]
    pub fn new() -> Self {
        Self { counter: 1 }
    }
}

impl Default for NumberSignIdProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SignIdProvider for NumberSignIdProvider {
    fn set_current_max_sign_id(&mut self, max_sign_id: &str) {
        if let Ok(num) = max_sign_id.parse::<u32>() {
            // counter 表示"下一个待分配的 ID"
            self.counter = num + 1;
        }
    }

    /// 分配并返回下一个签章 ID，内部计数器自增。
    fn increment_and_get(&mut self) -> String {
        self.counter += 1;
        self.counter.to_string()
    }

    /// 预览下一个待分配的签章 ID，不改变计数器。
    fn get(&self) -> String {
        self.counter.to_string()
    }

    fn parse_id(&self, id: &str) -> Result<u32, String> {
        id.parse::<u32>()
            .map_err(|e| format!("无法解析签章 ID: {e}"))
    }
}

/// 从 `sNNN` 格式的 ID 中解析数字部分。
fn parse_sign_id_number(id: &str) -> Option<u32> {
    let stripped = id.strip_prefix('s').unwrap_or(id);
    stripped.parse::<u32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_sign_id_provider_new() {
        let provider = StandardSignIdProvider::new();
        assert_eq!(provider.get(), "s001");
    }

    #[test]
    fn test_standard_sign_id_provider_increment() {
        let mut provider = StandardSignIdProvider::new();
        assert_eq!(provider.get(), "s001");
        assert_eq!(provider.increment_and_get(), "s002");
        assert_eq!(provider.increment_and_get(), "s003");
        assert_eq!(provider.get(), "s004");
    }

    #[test]
    fn test_standard_sign_id_provider_set_max() {
        let mut provider = StandardSignIdProvider::new();
        provider.set_current_max_sign_id("s005");
        assert_eq!(provider.get(), "s006");
        assert_eq!(provider.increment_and_get(), "s007");
    }

    #[test]
    fn test_standard_sign_id_provider_parse() {
        let provider = StandardSignIdProvider::new();
        assert_eq!(provider.parse_id("s001").unwrap(), 1);
        assert_eq!(provider.parse_id("s010").unwrap(), 10);
        assert!(provider.parse_id("abc").is_err());
    }

    #[test]
    fn test_standard_sign_id_provider_width() {
        let provider = StandardSignIdProvider::new().with_width(5);
        assert_eq!(provider.get(), "s00001");
    }

    #[test]
    fn test_number_sign_id_provider() {
        let mut provider = NumberSignIdProvider::new();
        assert_eq!(provider.get(), "1");
        assert_eq!(provider.increment_and_get(), "2");
        assert_eq!(provider.increment_and_get(), "3");
    }

    #[test]
    fn test_number_sign_id_provider_set_max() {
        let mut provider = NumberSignIdProvider::new();
        provider.set_current_max_sign_id("10");
        assert_eq!(provider.get(), "11");
    }

    #[test]
    fn test_parse_sign_id_number() {
        assert_eq!(parse_sign_id_number("s001"), Some(1));
        assert_eq!(parse_sign_id_number("s010"), Some(10));
        assert_eq!(parse_sign_id_number("s100"), Some(100));
        assert_eq!(parse_sign_id_number("abc"), None);
    }
}
