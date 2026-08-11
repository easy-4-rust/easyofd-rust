//! OFD-A 转换器。
//!
//! 对应 Java: org.ofdrw.archive.convert.OFDArchiveConverter

use super::archive_handler::ArchiveHandler;

/// OFD-A 转换器。
///
/// 将普通 OFD 文件转换为符合 GB/T 42133-2022 的 OFD-A 归档文件。
///
/// 对应 Java: org.ofdrw.archive.convert.OFDArchiveConverter
pub struct OfdArchiveConverter {
    /// 处理器管道，按顺序执行。
    handlers: Vec<Box<dyn ArchiveHandler>>,
}

impl OfdArchiveConverter {
    /// 创建转换器，使用自定义处理器管道。
    pub fn new(handlers: Vec<Box<dyn ArchiveHandler>>) -> Self {
        Self { handlers }
    }

    /// 执行全部处理器。
    ///
    /// 按顺序执行所有注册的处理器，单个处理器失败会中断流程。
    pub fn convert(&self, entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        for handler in &self.handlers {
            handler.handle(entries)?;
        }
        Ok(())
    }

    /// 获取已注册的处理器数量。
    #[must_use]
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for OfdArchiveConverter {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHandler {
        name: &'static str,
    }

    impl ArchiveHandler for MockHandler {
        fn name(&self) -> &'static str {
            self.name
        }

        fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
            Ok(())
        }
    }

    struct FailHandler;

    impl ArchiveHandler for FailHandler {
        fn name(&self) -> &'static str {
            "FailHandler"
        }

        fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
            Err("处理失败".into())
        }
    }

    #[test]
    fn converter_default_is_empty() {
        let converter = OfdArchiveConverter::default();
        assert_eq!(converter.handler_count(), 0);
    }

    #[test]
    fn converter_with_handlers() {
        let converter = OfdArchiveConverter::new(vec![
            Box::new(MockHandler { name: "A" }),
            Box::new(MockHandler { name: "B" }),
        ]);
        assert_eq!(converter.handler_count(), 2);
        let mut entries = vec![];
        assert!(converter.convert(&mut entries).is_ok());
    }

    #[test]
    fn converter_stops_on_failure() {
        let converter = OfdArchiveConverter::new(vec![
            Box::new(MockHandler { name: "A" }),
            Box::new(FailHandler),
            Box::new(MockHandler { name: "C" }),
        ]);
        let mut entries = vec![];
        assert!(converter.convert(&mut entries).is_err());
    }
}
