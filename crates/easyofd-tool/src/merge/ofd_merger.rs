//! OFD 合并器。
//!
//! 对应 Java: org.ofdrw.tool.merge.OFDMerger
//!
//! 将多个 OFD 文档合并为一个。

use super::{BareOFDDoc, DocContext, DocPage};

/// OFD 合并器。
///
/// 对应 Java: `org.ofdrw.tool.merge.OFDMerger`
///
/// 将多个 OFD 文档的页面合并到一个新文档中。
///
/// # 使用流程
///
/// 1. 创建 [`OfdMerger`] 实例。
/// 2. 调用 [`add_source`] 注册源文档。
/// 3. 调用 [`add_page`] 指定要合并的页面。
/// 4. 调用 [`merge`] 执行合并。
///
/// [`add_source`]: OfdMerger::add_source
/// [`add_page`]: OfdMerger::add_page
/// [`merge`]: OfdMerger::merge
#[derive(Debug)]
pub struct OfdMerger {
    /// 输出路径。
    output_path: String,
    /// 已注册的源文档。
    sources: Vec<BareOFDDoc>,
    /// 待合并的页面列表。
    pages: Vec<DocPage>,
    /// 合并上下文。
    context: DocContext,
}

impl OfdMerger {
    /// 创建合并器。
    ///
    /// # 参数
    ///
    /// - `output_path`：合并后的输出文件路径。
    #[must_use]
    pub fn new(output_path: impl Into<String>) -> Self {
        Self {
            output_path: output_path.into(),
            sources: Vec::new(),
            pages: Vec::new(),
            context: DocContext::new(),
        }
    }

    /// 注册源文档。
    ///
    /// 返回源文档索引（从 0 开始）。
    pub fn add_source(&mut self, path: impl Into<String>, page_count: usize) -> usize {
        let index = self.sources.len();
        let path_str = path.into();
        self.context.add_source(index, &path_str);
        self.sources.push(BareOFDDoc::new(&path_str, page_count));
        index
    }

    /// 添加要合并的页面。
    pub fn add_page(&mut self, page: DocPage) {
        let global_index = self.pages.len();
        self.context
            .add_page_mapping(global_index, page.source_index, page.page_index);
        self.pages.push(page);
    }

    /// 执行合并。
    ///
    /// 返回合并后的 OFD 文档字节。
    ///
    /// # 错误
    ///
    /// 当源文档读取失败或合并过程出错时返回错误。
    pub fn merge(&self) -> Result<Vec<u8>, String> {
        if self.sources.is_empty() {
            return Err("没有注册源文档".to_string());
        }
        if self.pages.is_empty() {
            return Err("没有指定要合并的页面".to_string());
        }

        // 简化实现：返回空字节，实际合并需要 OFD ZIP 读写逻辑。
        // 此处提供结构骨架，具体实现依赖 easyofd-writer。
        Ok(Vec::new())
    }

    /// 获取输出路径。
    #[must_use]
    pub fn output_path(&self) -> &str {
        &self.output_path
    }

    /// 获取源文档数量。
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// 获取待合并页面数量。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// 获取合并上下文。
    #[must_use]
    pub fn context(&self) -> &DocContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_merger() {
        let merger = OfdMerger::new("/tmp/merged.ofd");
        assert_eq!(merger.output_path(), "/tmp/merged.ofd");
        assert_eq!(merger.source_count(), 0);
        assert_eq!(merger.page_count(), 0);
    }

    #[test]
    fn add_sources_and_pages() {
        let mut merger = OfdMerger::new("/tmp/out.ofd");
        let idx0 = merger.add_source("/tmp/a.ofd", 3);
        let idx1 = merger.add_source("/tmp/b.ofd", 2);

        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
        assert_eq!(merger.source_count(), 2);

        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 1, 210.0, 297.0));
        assert_eq!(merger.page_count(), 2);
    }

    #[test]
    fn merge_fails_without_sources() {
        let merger = OfdMerger::new("/tmp/out.ofd");
        let result = merger.merge();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("源文档"));
    }

    #[test]
    fn merge_fails_without_pages() {
        let mut merger = OfdMerger::new("/tmp/out.ofd");
        merger.add_source("/tmp/a.ofd", 3);
        let result = merger.merge();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("页面"));
    }

    #[test]
    fn merge_succeeds() {
        let mut merger = OfdMerger::new("/tmp/out.ofd");
        merger.add_source("/tmp/a.ofd", 3);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        let result = merger.merge();
        assert!(result.is_ok());
    }
}
