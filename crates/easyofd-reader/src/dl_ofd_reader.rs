//! DLOFD 解析器（废弃）。
//!
//! 对应 Java: org.ofdrw.reader.DLOFDReader
//!
//! 已废弃：Java 原始类标记为 `@Deprecated`，建议使用 `OFDReader`。
//! Rust 版提供轻量门面，委托给 `OfdReaderFacade`。

use std::io::{Read, Seek};

use easyofd_core::{OfdPage, OfdResult};

#[allow(deprecated)]
use crate::model::{AnnotionEntity, OfdPageVo};
use crate::ofd_reader_facade::OfdReaderFacade;
use crate::read_options::ReadOptions;

/// DLOFD 解析器。
///
/// 对应 Java: `org.ofdrw.reader.DLOFDReader`
///
/// **废弃**：Java 原始类标记为 `@Deprecated`，建议使用 [`OfdReaderFacade`]。
///
/// 提供与 Java 版兼容的 API，内部委托给 [`OfdReaderFacade`]。
#[derive(Debug)]
#[deprecated(since = "1.0.0", note = "使用 OfdReaderFacade 替代")]
pub struct DlOfdReader {
    /// 内部门面。
    facade: OfdReaderFacade,
}

#[allow(deprecated)]
impl DlOfdReader {
    /// 从文件路径打开 OFD 文档。
    ///
    /// 对应 Java: `DLOFDReader(Path ofdFile)`
    pub fn open(path: impl AsRef<std::path::Path>) -> OfdResult<Self> {
        Ok(Self {
            facade: OfdReaderFacade::open(path)?,
        })
    }

    /// 从字节数组解析 OFD 文档。
    ///
    /// 对应 Java: `DLOFDReader(InputStream stream)`
    pub fn from_bytes(data: &[u8]) -> OfdResult<Self> {
        Ok(Self {
            facade: OfdReaderFacade::from_bytes(data)?,
        })
    }

    /// 从 `Read + Seek` 输入创建。
    pub fn from_seek<R: Read + Seek>(source: R, options: ReadOptions) -> OfdResult<Self> {
        Ok(Self {
            facade: OfdReaderFacade::from_seek(source, options)?,
        })
    }

    /// 获取内部门面引用。
    #[must_use]
    pub fn facade(&self) -> &OfdReaderFacade {
        &self.facade
    }

    /// 获取文档页数。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.facade.page_count()
    }

    /// 获取所有页面。
    #[must_use]
    pub fn pages(&self) -> &[OfdPage] {
        self.facade.pages()
    }

    /// 获取 OFD 页面视图对象列表。
    ///
    /// 对应 Java: `DLOFDReader.getOFDPageVO()`
    #[must_use]
    pub fn get_ofd_page_vo(&self) -> Vec<OfdPageVo> {
        self.facade
            .pages()
            .iter()
            .enumerate()
            .map(|(i, page)| {
                let page_path = page
                    .base_path
                    .clone()
                    .unwrap_or_else(|| format!("Pages/Page_{i}/Content.xml"));
                OfdPageVo::new(page_path, None)
            })
            .collect()
    }

    /// 获取注释列表（简化实现）。
    ///
    /// 对应 Java: `DLOFDReader.getAnnotaions()`
    #[must_use]
    pub fn get_annotations(&self) -> Vec<AnnotionEntity> {
        // 简化实现：返回空列表，因为注释解析需要完整的 XML 模型
        Vec::new()
    }

    /// 提取所有文本。
    #[must_use]
    pub fn extract_all_text(&self) -> String {
        self.facade.extract_all_text()
    }
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;
    use easyofd_core::TextObject;
    use easyofd_writer::OfdWriter;

    fn build_test_ofd() -> Vec<u8> {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "DL Test"));
        writer.add_page(page);
        writer.build().unwrap()
    }

    #[test]
    #[allow(deprecated)]
    fn test_dl_ofd_reader_from_bytes() {
        let bytes = build_test_ofd();
        let reader = DlOfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 1);
    }

    #[test]
    #[allow(deprecated)]
    fn test_dl_ofd_reader_get_ofd_page_vo() {
        let bytes = build_test_ofd();
        let reader = DlOfdReader::from_bytes(&bytes).unwrap();
        let vos = reader.get_ofd_page_vo();
        assert_eq!(vos.len(), 1);
    }

    #[test]
    #[allow(deprecated)]
    fn test_dl_ofd_reader_annotations() {
        let bytes = build_test_ofd();
        let reader = DlOfdReader::from_bytes(&bytes).unwrap();
        assert!(reader.get_annotations().is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn test_dl_ofd_reader_extract_text() {
        let bytes = build_test_ofd();
        let reader = DlOfdReader::from_bytes(&bytes).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("DL Test"));
    }
}
