//! 合并上下文。
//!
//! 对应 Java: org.ofdrw.tool.merge.DocContext
//!
//! 在 OFD 文档合并过程中维护源文档和目标文档的上下文信息。

use std::collections::HashMap;

/// 合并上下文。
///
/// 对应 Java: `org.ofdrw.tool.merge.DocContext`
///
/// 维护合并过程中的状态信息，包括源文档路径映射、
/// 页面索引重映射、资源文件路径映射等。
#[derive(Debug, Default)]
pub struct DocContext {
    /// 源文档索引 → 源文档路径。
    source_paths: HashMap<usize, String>,
    /// 源页面全局索引 → (源文档索引, 源页面索引)。
    page_mapping: HashMap<usize, (usize, usize)>,
    /// 资源路径重映射（源路径 → 目标路径）。
    resource_remap: HashMap<String, String>,
}

impl DocContext {
    /// 创建空上下文。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册源文档。
    pub fn add_source(&mut self, index: usize, path: impl Into<String>) {
        self.source_paths.insert(index, path.into());
    }

    /// 获取源文档路径。
    #[must_use]
    pub fn source_path(&self, index: usize) -> Option<&str> {
        self.source_paths.get(&index).map(|s| s.as_str())
    }

    /// 注册页面映射。
    pub fn add_page_mapping(
        &mut self,
        global_index: usize,
        source_index: usize,
        page_index: usize,
    ) {
        self.page_mapping
            .insert(global_index, (source_index, page_index));
    }

    /// 获取页面映射。
    #[must_use]
    pub fn page_mapping(&self, global_index: usize) -> Option<(usize, usize)> {
        self.page_mapping.get(&global_index).copied()
    }

    /// 注册资源路径重映射。
    pub fn add_resource_remap(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.resource_remap.insert(from.into(), to.into());
    }

    /// 获取资源路径重映射。
    #[must_use]
    pub fn resource_remap(&self, from: &str) -> Option<&str> {
        self.resource_remap.get(from).map(|s| s.as_str())
    }

    /// 获取源文档数量。
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.source_paths.len()
    }

    /// 获取页面映射数量。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.page_mapping.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let ctx = DocContext::new();
        assert_eq!(ctx.source_count(), 0);
        assert_eq!(ctx.page_count(), 0);
    }

    #[test]
    fn add_source_and_get() {
        let mut ctx = DocContext::new();
        ctx.add_source(0, "/tmp/doc1.ofd");
        ctx.add_source(1, "/tmp/doc2.ofd");

        assert_eq!(ctx.source_count(), 2);
        assert_eq!(ctx.source_path(0), Some("/tmp/doc1.ofd"));
        assert_eq!(ctx.source_path(1), Some("/tmp/doc2.ofd"));
        assert!(ctx.source_path(2).is_none());
    }

    #[test]
    fn page_mapping_roundtrip() {
        let mut ctx = DocContext::new();
        ctx.add_page_mapping(0, 0, 0);
        ctx.add_page_mapping(1, 0, 1);
        ctx.add_page_mapping(2, 1, 0);

        assert_eq!(ctx.page_mapping(0), Some((0, 0)));
        assert_eq!(ctx.page_mapping(1), Some((0, 1)));
        assert_eq!(ctx.page_mapping(2), Some((1, 0)));
        assert_eq!(ctx.page_count(), 3);
    }

    #[test]
    fn resource_remap() {
        let mut ctx = DocContext::new();
        ctx.add_resource_remap("res/font.ttf", "res/font_0.ttf");

        assert_eq!(ctx.resource_remap("res/font.ttf"), Some("res/font_0.ttf"));
        assert!(ctx.resource_remap("res/other.ttf").is_none());
    }
}
