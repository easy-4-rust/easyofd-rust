//! 导入器抽象 trait。

use std::path::Path;

use easyofd_core::OfdResult;

/// 通用导入接口。
///
/// 对应 Java: org.ofdrw.converter.importer.Importer
///
/// 所有导入器（PDF→OFD 等）都实现此 trait。
pub trait Importer {
    /// 将源文件转换为 OFD 格式并写入指定路径。
    ///
    /// # 参数
    ///
    /// - `source`: 源文件路径
    /// - `target`: 目标 OFD 文件路径
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

    /// 测试用的简单导入器实现。
    struct TestImporter;

    impl Importer for TestImporter {
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
    fn test_importer_trait_exists() {
        // 验证 trait 可以被实现
        let importer = TestImporter;
        assert!(
            importer
                .convert(
                    PathBuf::from("/nonexistent").as_path(),
                    PathBuf::from("/tmp/out").as_path()
                )
                .is_err()
        );
    }

    #[test]
    fn test_importer_convert_missing_source() {
        let importer = TestImporter;
        let result = importer.convert(
            &PathBuf::from("/nonexistent/source.pdf"),
            &PathBuf::from("/tmp/out.ofd"),
        );
        assert!(result.is_err());
    }
}
