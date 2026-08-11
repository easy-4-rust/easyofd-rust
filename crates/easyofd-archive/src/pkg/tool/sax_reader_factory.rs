//! SAX 读取器工厂。
//!
//! 对应 Java: org.ofdrw.pkg.tool.SAXReaderFactory
//!
//! 提供 XML SAX 读取器的创建工厂。
//! 在 Rust 生态中，通常使用 quick-xml 或其他 XML 解析库，
//! 此类型提供统一的配置入口。

/// SAX 读取器工厂。
///
/// 对应 Java: `org.ofdrw.pkg.tool.SAXReaderFactory`
///
/// 用于创建配置好的 XML 读取器实例。
/// Java 版本使用 dom4j SAXReader，Rust 版本使用 quick-xml。
///
/// 此类型为简化结构，记录解析器配置。
#[derive(Debug, Clone)]
pub struct SaxReaderFactory {
    /// 是否验证 XML 命名空间。
    validate_namespace: bool,
    /// 是否忽略注释。
    ignore_comments: bool,
}

impl SaxReaderFactory {
    /// 创建默认工厂。
    #[must_use]
    pub fn new() -> Self {
        Self {
            validate_namespace: true,
            ignore_comments: true,
        }
    }

    /// 是否验证命名空间。
    #[must_use]
    pub fn validate_namespace(&self) -> bool {
        self.validate_namespace
    }

    /// 设置是否验证命名空间。
    pub fn set_validate_namespace(&mut self, validate: bool) {
        self.validate_namespace = validate;
    }

    /// 是否忽略注释。
    #[must_use]
    pub fn ignore_comments(&self) -> bool {
        self.ignore_comments
    }

    /// 设置是否忽略注释。
    pub fn set_ignore_comments(&mut self, ignore: bool) {
        self.ignore_comments = ignore;
    }
}

impl Default for SaxReaderFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let factory = SaxReaderFactory::new();
        assert!(factory.validate_namespace());
        assert!(factory.ignore_comments());
    }

    #[test]
    fn setters() {
        let mut factory = SaxReaderFactory::new();
        factory.set_validate_namespace(false);
        factory.set_ignore_comments(false);
        assert!(!factory.validate_namespace());
        assert!(!factory.ignore_comments());
    }
}
