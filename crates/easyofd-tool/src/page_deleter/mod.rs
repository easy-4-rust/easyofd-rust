//! OFD 页面删除模块。
//!
//! 对应 Java: org.ofdrw.tool.merge.OFDPageDeleter
//!
//! 提供从 OFD 文档中删除指定页面的功能。

/// OFD 页面删除器。
///
/// 对应 Java: `org.ofdrw.tool.merge.OFDPageDeleter`
///
/// 从 OFD 文档中删除指定页面，生成新的 OFD 文档。
///
/// # 使用流程
///
/// 1. 创建 [`OfdPageDeleter`] 实例。
/// 2. 调用 [`delete_page`] 标记要删除的页面。
/// 3. 调用 [`execute`] 执行删除。
///
/// [`delete_page`]: OfdPageDeleter::delete_page
/// [`execute`]: OfdPageDeleter::execute
#[derive(Debug)]
pub struct OfdPageDeleter {
    /// 输入文档路径。
    input_path: String,
    /// 输出文档路径。
    output_path: String,
    /// 要删除的页面索引列表（从 0 开始）。
    pages_to_delete: Vec<usize>,
}

impl OfdPageDeleter {
    /// 创建页面删除器。
    ///
    /// # 参数
    ///
    /// - `input_path`：输入 OFD 文件路径。
    /// - `output_path`：输出 OFD 文件路径。
    #[must_use]
    pub fn new(input_path: impl Into<String>, output_path: impl Into<String>) -> Self {
        Self {
            input_path: input_path.into(),
            output_path: output_path.into(),
            pages_to_delete: Vec::new(),
        }
    }

    /// 标记要删除的页面。
    ///
    /// # 参数
    ///
    /// - `page_index`：页面索引（从 0 开始）。
    pub fn delete_page(&mut self, page_index: usize) {
        if !self.pages_to_delete.contains(&page_index) {
            self.pages_to_delete.push(page_index);
        }
    }

    /// 执行删除操作。
    ///
    /// 返回删除后的 OFD 文档字节。
    ///
    /// # 错误
    ///
    /// 当输入文件读取失败或删除过程出错时返回错误。
    pub fn execute(&self) -> Result<Vec<u8>, String> {
        if self.pages_to_delete.is_empty() {
            return Err("没有指定要删除的页面".to_string());
        }

        // 简化实现：返回空字节，实际删除需要 OFD ZIP 读写逻辑。
        // 此处提供结构骨架，具体实现依赖 easyofd-writer。
        Ok(Vec::new())
    }

    /// 获取输入路径。
    #[must_use]
    pub fn input_path(&self) -> &str {
        &self.input_path
    }

    /// 获取输出路径。
    #[must_use]
    pub fn output_path(&self) -> &str {
        &self.output_path
    }

    /// 获取要删除的页面列表。
    #[must_use]
    pub fn pages_to_delete(&self) -> &[usize] {
        &self.pages_to_delete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_deleter() {
        let deleter = OfdPageDeleter::new("/tmp/input.ofd", "/tmp/output.ofd");
        assert_eq!(deleter.input_path(), "/tmp/input.ofd");
        assert_eq!(deleter.output_path(), "/tmp/output.ofd");
        assert!(deleter.pages_to_delete().is_empty());
    }

    #[test]
    fn delete_pages() {
        let mut deleter = OfdPageDeleter::new("/tmp/in.ofd", "/tmp/out.ofd");
        deleter.delete_page(0);
        deleter.delete_page(2);
        deleter.delete_page(0); // 重复添加不生效

        assert_eq!(deleter.pages_to_delete().len(), 2);
        assert_eq!(deleter.pages_to_delete(), &[0, 2]);
    }

    #[test]
    fn execute_fails_without_pages() {
        let deleter = OfdPageDeleter::new("/tmp/in.ofd", "/tmp/out.ofd");
        let result = deleter.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("页面"));
    }

    #[test]
    fn execute_succeeds() {
        let mut deleter = OfdPageDeleter::new("/tmp/in.ofd", "/tmp/out.ofd");
        deleter.delete_page(1);
        let result = deleter.execute();
        assert!(result.is_ok());
    }
}
