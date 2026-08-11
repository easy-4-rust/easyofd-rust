//! 导出器抽象 trait。

use std::path::Path;

use easyofd_core::OfdResult;

/// 通用导出接口。
///
/// 对应 Java: org.ofdrw.converter.exporter.Exporter
///
/// 所有导出器（OFD→PDF、OFD→PNG 等）都实现此 trait。
pub trait Exporter {
    /// 将源文件转换为目标格式并写入指定路径。
    ///
    /// # 参数
    ///
    /// - `source`: 源文件路径
    /// - `target`: 目标文件路径
    ///
    /// # 错误
    ///
    /// 如果转换失败则返回错误。
    fn convert(&self, source: &Path, target: &Path) -> OfdResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// 测试用的简单导出器实现。
    struct TestExporter;

    impl Exporter for TestExporter {
        fn convert(&self, source: &Path, target: &Path) -> OfdResult<()> {
            if !source.exists() {
                return Err(easyofd_core::OfdError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "源文件不存在",
                )));
            }
            // 简单复制
            std::fs::copy(source, target).map_err(easyofd_core::OfdError::Io)?;
            Ok(())
        }
    }

    #[test]
    fn test_exporter_trait_exists() {
        // 验证 trait 可以被实现
        let exporter = TestExporter;
        assert!(
            exporter
                .convert(Path::new("/nonexistent"), Path::new("/tmp/out"))
                .is_err()
        );
    }

    #[test]
    fn test_exporter_convert_missing_source() {
        let exporter = TestExporter;
        let result = exporter.convert(
            &PathBuf::from("/nonexistent/source.txt"),
            &PathBuf::from("/tmp/out.txt"),
        );
        assert!(result.is_err());
    }
}
