//! 内容抽取器。
//!
//! 对应 Java: org.ofdrw.reader.ContentExtractor
//!
//! 从 OFD 文档中抽取文本内容，支持可选的过滤器。

use easyofd_core::OfdPage;

/// 文本抽取过滤器函数类型。
///
/// 对应 Java: `org.ofdrw.reader.extractor.ExtractorFilter`
///
/// 接收文本内容，返回是否允许该文本通过过滤。
/// 返回 `None` 表示过滤掉该文本，返回 `Some(text)` 表示允许。
pub type ExtractorFilterFn = dyn Fn(&str) -> Option<String>;

/// 内容抽取器，从已解析的 OFD 页面中抽取文本。
///
/// 对应 Java: `org.ofdrw.reader.ContentExtractor`
///
/// # 用法
///
/// ```rust,no_run
/// use easyofd_reader::ContentExtractor;
///
/// # fn example(pages: &[easyofd_core::OfdPage]) {
/// let extractor = ContentExtractor::new();
/// let texts = extractor.get_page_content(pages, 1);
/// # }
/// ```
#[derive(Default)]
pub struct ContentExtractor {
    /// 文本抽取过滤器。
    filter: Option<Box<ExtractorFilterFn>>,
}

/// 文本抽取结果接收器的回调函数类型。
///
/// 对应 Java: `ContentExtractor.Receiver`
pub type ReceiverFn = dyn FnMut(usize, &[String]);

impl ContentExtractor {
    /// 创建不带过滤器的内容抽取器。
    ///
    /// 对应 Java: `ContentExtractor(OFDReader)`
    #[must_use]
    pub fn new() -> Self {
        Self { filter: None }
    }

    /// 创建带过滤器的内容抽取器。
    ///
    /// 对应 Java: `ContentExtractor(OFDReader, ExtractorFilter)`
    #[must_use]
    pub fn with_filter(filter: Box<ExtractorFilterFn>) -> Self {
        Self {
            filter: Some(filter),
        }
    }

    /// 抽取指定页面内的所有文字。
    ///
    /// 对应 Java: `ContentExtractor.getPageContent(int pageNum)`
    ///
    /// `page_num` 从 1 开始。
    #[must_use]
    pub fn get_page_content(&self, pages: &[OfdPage], page_num: usize) -> Vec<String> {
        if page_num == 0 || page_num > pages.len() {
            return Vec::new();
        }
        let page = &pages[page_num - 1];
        self.extract_page_texts(page)
    }

    /// 获取 OFD 内所有页面的文本内容。
    ///
    /// 对应 Java: `ContentExtractor.extractAll()`
    #[must_use]
    pub fn extract_all(&self, pages: &[OfdPage]) -> Vec<String> {
        let mut all_texts = Vec::new();
        for page in pages {
            let page_texts = self.extract_page_texts(page);
            all_texts.extend(page_texts);
        }
        all_texts
    }

    /// 遍历所有页面，对每页文本调用回调。
    ///
    /// 对应 Java: `ContentExtractor.traverse(Receiver)`
    pub fn traverse(&self, pages: &[OfdPage], mut receiver: impl FnMut(usize, &[String])) {
        for (i, page) in pages.iter().enumerate() {
            let texts = self.extract_page_texts(page);
            if !texts.is_empty() {
                receiver(i + 1, &texts);
            }
        }
    }

    /// 从单个页面提取文本。
    fn extract_page_texts(&self, page: &OfdPage) -> Vec<String> {
        use easyofd_core::ContentObject;

        let mut texts = Vec::new();
        for obj in &page.content {
            if let ContentObject::Text(t) = obj {
                let text = &t.text;
                if let Some(ref filter) = self.filter {
                    if let Some(allowed) = filter(text) {
                        if !allowed.is_empty() {
                            texts.push(allowed);
                        }
                    }
                } else if !text.is_empty() {
                    texts.push(text.clone());
                }
            }
        }
        texts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::TextObject;

    fn make_page_with_texts(texts: &[&str]) -> OfdPage {
        let mut page = OfdPage::new(210.0, 297.0);
        for text in texts {
            page.add_text(TextObject::new(0.0, 0.0, *text));
        }
        page
    }

    #[test]
    fn test_extract_single_page() {
        let pages = vec![make_page_with_texts(&["Hello", "World"])];
        let extractor = ContentExtractor::new();
        let result = extractor.get_page_content(&pages, 1);
        assert_eq!(result, vec!["Hello", "World"]);
    }

    #[test]
    fn test_extract_page_out_of_range() {
        let pages = vec![make_page_with_texts(&["Hello"])];
        let extractor = ContentExtractor::new();
        assert!(extractor.get_page_content(&pages, 0).is_empty());
        assert!(extractor.get_page_content(&pages, 2).is_empty());
    }

    #[test]
    fn test_extract_all() {
        let pages = vec![
            make_page_with_texts(&["Page1"]),
            make_page_with_texts(&["Page2"]),
        ];
        let extractor = ContentExtractor::new();
        let result = extractor.extract_all(&pages);
        assert_eq!(result, vec!["Page1", "Page2"]);
    }

    #[test]
    fn test_with_filter() {
        let pages = vec![make_page_with_texts(&["Hello", "SECRET", "World"])];
        let extractor = ContentExtractor::with_filter(Box::new(|text| {
            if text.contains("SECRET") {
                None
            } else {
                Some(text.to_string())
            }
        }));
        let result = extractor.get_page_content(&pages, 1);
        assert_eq!(result, vec!["Hello", "World"]);
    }

    #[test]
    fn test_traverse() {
        let pages = vec![make_page_with_texts(&["A"]), make_page_with_texts(&["B"])];
        let extractor = ContentExtractor::new();
        let mut visited = Vec::new();
        extractor.traverse(&pages, |page_num, texts| {
            visited.push((page_num, texts.to_vec()));
        });
        assert_eq!(visited.len(), 2);
        assert_eq!(visited[0].0, 1);
        assert_eq!(visited[1].0, 2);
    }
}
