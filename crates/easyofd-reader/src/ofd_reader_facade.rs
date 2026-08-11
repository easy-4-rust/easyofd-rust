//! OFD 解析器门面。
//!
//! 对应 Java: org.ofdrw.reader.OFDReader
//!
//! Java 版 `OFDReader` 是一个功能丰富的门面类，管理临时目录解压、
//! 资源定位器、资源管理器等。Rust 版已有的 `OfdReader` 实现了核心
//! 解析功能，此门面提供额外的资源定位和管理能力。

use std::io::{Read, Seek};

use easyofd_core::{OfdPage, OfdResult};

use crate::OfdReader;
use crate::page_info::PageInfo;
use crate::read_options::ReadOptions;
use crate::resource_locator::ResourceLocator;
use crate::resource_manage::ResourceManage;

/// OFD 解析器门面，提供完整的 OFD 文档读取能力。
///
/// 对应 Java: `org.ofdrw.reader.OFDReader`
///
/// 封装 [`OfdReader`] 的核心解析能力，额外提供：
/// - [`ResourceLocator`] 资源路径定位
/// - [`ResourceManage`] 资源 ID 索引
/// - [`PageInfo`] 页面元数据访问
///
/// # 用法
///
/// ```rust,no_run
/// use easyofd_reader::OfdReaderFacade;
///
/// # fn example() -> Result<(), easyofd_core::OfdError> {
/// let facade = OfdReaderFacade::open("document.ofd")?;
/// println!("页数: {}", facade.page_count());
/// # Ok(())
/// # }
/// ```
pub struct OfdReaderFacade {
    /// 内部 OFD 读取器。
    reader: OfdReader,
    /// 资源定位器。
    locator: ResourceLocator,
    /// 资源管理器（懒加载）。
    resource_manage: Option<ResourceManage>,
}

impl std::fmt::Debug for OfdReaderFacade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OfdReaderFacade")
            .field("page_count", &self.reader.page_count())
            .field("locator", &self.locator)
            .field("resource_manage", &self.resource_manage.is_some())
            .finish()
    }
}

impl OfdReaderFacade {
    /// 从文件路径打开 OFD 文档。
    ///
    /// 对应 Java: `OFDReader(Path ofdFile)`
    ///
    /// # 错误
    ///
    /// 文件无法读取或 OFD 数据无效时返回错误。
    pub fn open(path: impl AsRef<std::path::Path>) -> OfdResult<Self> {
        let reader = OfdReader::open(path)?;
        Ok(Self::from_reader(reader))
    }

    /// 从字节数组解析 OFD 文档。
    ///
    /// 对应 Java: `OFDReader(InputStream stream)`
    ///
    /// # 错误
    ///
    /// 数据无效时返回错误。
    pub fn from_bytes(data: &[u8]) -> OfdResult<Self> {
        let reader = OfdReader::from_bytes(data)?;
        Ok(Self::from_reader(reader))
    }

    /// 从已有的 `OfdReader` 创建门面。
    #[must_use]
    pub fn from_reader(reader: OfdReader) -> Self {
        Self {
            reader,
            locator: ResourceLocator::new(),
            resource_manage: None,
        }
    }

    /// 从 `Read + Seek` 输入创建门面。
    ///
    /// # 错误
    ///
    /// ZIP 包或 XML 无效时返回错误。
    pub fn from_seek<R: Read + Seek>(source: R, options: ReadOptions) -> OfdResult<Self> {
        let reader = OfdReader::from_seek(source, options)?;
        Ok(Self::from_reader(reader))
    }

    /// 获取内部的 `OfdReader` 引用。
    #[must_use]
    pub fn reader(&self) -> &OfdReader {
        &self.reader
    }

    /// 获取资源定位器。
    ///
    /// 对应 Java: `OFDReader.getResourceLocator()`
    #[must_use]
    pub fn resource_locator(&self) -> &ResourceLocator {
        &self.locator
    }

    /// 获取可变资源定位器。
    #[must_use]
    pub fn resource_locator_mut(&mut self) -> &mut ResourceLocator {
        &mut self.locator
    }

    /// 获取文档页数。
    ///
    /// 对应 Java: `OFDReader.getNumberOfPages()`
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.reader.page_count()
    }

    /// 获取所有页面。
    #[must_use]
    pub fn pages(&self) -> &[OfdPage] {
        self.reader.pages()
    }

    /// 获取指定页面的信息。
    ///
    /// 对应 Java: `OFDReader.getPageInfo(int pageNum)`
    ///
    /// `page_num` 从 1 开始。
    #[must_use]
    pub fn get_page_info(&self, page_num: usize) -> Option<PageInfo> {
        if page_num == 0 || page_num > self.reader.page_count() {
            return None;
        }
        let page = &self.reader.pages()[page_num - 1];
        let size = easyofd_core::ST_Box::new(0.0, 0.0, page.width, page.height);
        let mut info = PageInfo::new(page_num, size);
        info.page_n = page_num.saturating_sub(1);
        Some(info)
    }

    /// 获取所有页面信息列表。
    ///
    /// 对应 Java: `OFDReader.getPageList()`
    #[must_use]
    pub fn get_page_list(&self) -> Vec<PageInfo> {
        (1..=self.page_count())
            .filter_map(|i| self.get_page_info(i))
            .collect()
    }

    /// 获取文档元数据。
    #[must_use]
    pub fn metadata(&self) -> &easyofd_core::OfdMetadata {
        self.reader.metadata()
    }

    /// 提取所有页面的文本。
    #[must_use]
    pub fn extract_text(&self) -> Vec<String> {
        self.reader.extract_text()
    }

    /// 提取所有文本合并为单个字符串。
    #[must_use]
    pub fn extract_all_text(&self) -> String {
        self.reader.extract_all_text()
    }

    /// 获取资源管理器。
    ///
    /// 对应 Java: `OFDReader.getResMgt()`
    #[must_use]
    pub fn resource_manage(&self) -> Option<&ResourceManage> {
        self.resource_manage.as_ref()
    }

    /// 设置资源管理器。
    pub fn set_resource_manage(&mut self, mgr: ResourceManage) {
        self.resource_manage = Some(mgr);
    }

    /// 文档是否包含数字签名。
    ///
    /// 对应 Java: `OFDReader.hasSignature()`
    #[must_use]
    pub fn has_signature(&self) -> bool {
        self.reader.metadata().signatures_path.is_some()
    }

    /// 获取原始 ZIP 条目（用于无损 roundtrip）。
    #[must_use]
    pub fn raw_entries(&self) -> &[(String, Vec<u8>)] {
        self.reader.raw_entries()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;

    fn build_test_ofd() -> Vec<u8> {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "Facade Test"));
        writer.add_page(page);
        writer.build().unwrap()
    }

    #[test]
    fn test_facade_from_bytes() {
        let bytes = build_test_ofd();
        let facade = OfdReaderFacade::from_bytes(&bytes).unwrap();
        assert_eq!(facade.page_count(), 1);
        assert!(!facade.has_signature());
    }

    #[test]
    fn test_facade_from_reader() {
        let bytes = build_test_ofd();
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        let facade = OfdReaderFacade::from_reader(reader);
        assert_eq!(facade.page_count(), 1);
    }

    #[test]
    fn test_facade_get_page_info() {
        let bytes = build_test_ofd();
        let facade = OfdReaderFacade::from_bytes(&bytes).unwrap();
        let info = facade.get_page_info(1).unwrap();
        assert_eq!(info.index, 1);
        assert!((info.width() - 210.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_facade_get_page_info_out_of_range() {
        let bytes = build_test_ofd();
        let facade = OfdReaderFacade::from_bytes(&bytes).unwrap();
        assert!(facade.get_page_info(0).is_none());
        assert!(facade.get_page_info(2).is_none());
    }

    #[test]
    fn test_facade_get_page_list() {
        let bytes = build_test_ofd();
        let facade = OfdReaderFacade::from_bytes(&bytes).unwrap();
        let list = facade.get_page_list();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_facade_extract_text() {
        let bytes = build_test_ofd();
        let facade = OfdReaderFacade::from_bytes(&bytes).unwrap();
        let texts = facade.extract_text();
        assert!(!texts.is_empty());
        assert!(texts[0].contains("Facade Test"));
    }

    #[test]
    fn test_facade_resource_locator() {
        let bytes = build_test_ofd();
        let mut facade = OfdReaderFacade::from_bytes(&bytes).unwrap();
        facade.resource_locator_mut().cd("/Doc_0").unwrap();
        assert_eq!(facade.resource_locator().pwd(), "/Doc_0");
    }

    #[test]
    fn test_facade_resource_manage() {
        let bytes = build_test_ofd();
        let mut facade = OfdReaderFacade::from_bytes(&bytes).unwrap();
        assert!(facade.resource_manage().is_none());
        facade.set_resource_manage(ResourceManage::new());
        assert!(facade.resource_manage().is_some());
    }
}
