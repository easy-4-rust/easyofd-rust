//! OFD 导出器 trait 别名。
//!
//! 对应 Java: org.ofdrw.converter.export.OFDExporter
//!
//! Java 版 `OFDExporter` 是一个接口，定义了 OFD 导出的通用方法。
//! Rust 版使用 [`super::Exporter`] trait 替代。
//!
//! 此模块提供 Java 接口名兼容的类型别名。

use std::path::Path;

use easyofd_core::OfdResult;

/// OFD 导出器 trait。
///
/// 对应 Java: `org.ofdrw.converter.export.OFDExporter`
///
/// Java 原始接口定义了 `export(Path source, Path target)` 方法。
/// Rust 版使用 [`super::Exporter`] trait，提供 `convert(&Path, &Path)` 方法。
///
/// 此类型别名保持与 Java API 的名称兼容。
pub trait OFDExporter {
    /// 将源 OFD 文件导出到目标路径。
    ///
    /// # 错误
    /// 导出失败时返回错误。
    fn export(&self, source: &Path, target: &Path) -> OfdResult<()>;
}

/// 为所有实现 [`super::Exporter`] 的类型自动实现 [`OFDExporter`]。
impl<T: super::Exporter> OFDExporter for T {
    fn export(&self, source: &Path, target: &Path) -> OfdResult<()> {
        self.convert(source, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExporter;

    impl super::super::Exporter for MockExporter {
        fn convert(&self, _source: &Path, _target: &Path) -> OfdResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_ofd_exporter_trait() {
        let exporter = MockExporter;
        // 通过 OFDExporter trait 调用
        let result = exporter.export(Path::new("/a"), Path::new("/b"));
        assert!(result.is_ok());
    }
}
