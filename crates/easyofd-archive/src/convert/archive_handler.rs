//! OFD-A 转换处理器 trait。
//!
//! 对应 Java: org.ofdrw.archive.convert.ArchiveHandler

/// OFD-A 转换处理器。
///
/// 每个处理器执行一个具体的"去技术化"操作，就地修改 OFD 容器中的文件。
/// 处理器按编排顺序依次执行。
/// 处理器实现应为无状态，允许多次调用。
///
/// 对应 Java: org.ofdrw.archive.convert.ArchiveHandler
pub trait ArchiveHandler: Send + Sync {
    /// 处理器名称。
    fn name(&self) -> &'static str;

    /// 执行去技术化转换操作。
    ///
    /// `entries` 是 OFD 包内所有文件的 `(路径, 内容)` 列表。
    /// 处理器可以直接修改 entries 中的内容。
    fn handle(&self, entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHandler;

    impl ArchiveHandler for MockHandler {
        fn name(&self) -> &'static str {
            "MockHandler"
        }

        fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn mock_handler_name() {
        let handler = MockHandler;
        assert_eq!(handler.name(), "MockHandler");
    }

    #[test]
    fn mock_handler_handle() {
        let handler = MockHandler;
        let mut entries = vec![];
        assert!(handler.handle(&mut entries).is_ok());
    }
}
