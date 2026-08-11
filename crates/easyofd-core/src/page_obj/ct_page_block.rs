//! CT_PageBlock 页面块容器。

/// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.block.CT_PageBlock
///
/// 页面块容器，可以嵌套。用于组织页面内容，支持包含文本对象、
/// 图像对象、路径对象以及嵌套的页面块。
/// 对应 GB/T 33190-2016 第 7.7 节图 17 表 16。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_PageBlock {
    /// 嵌套的页面块列表。
    pub page_blocks: Vec<CT_PageBlock>,
    /// 文本对象列表。
    pub text_objects: Vec<PageBlockTextObject>,
    /// 路径对象列表。
    pub path_objects: Vec<PageBlockPathObject>,
    /// 图像对象列表。
    pub image_objects: Vec<PageBlockImageObject>,
}

/// 页面块中的文本对象（简化表示）。
#[derive(Debug, Clone)]
pub struct PageBlockTextObject {
    /// 对象 ID。
    pub id: u32,
    /// 边界框 "x y width height"。
    pub boundary: String,
    /// 文本内容。
    pub content: String,
    /// 字号（pt）。
    pub font_size: f64,
}

/// 页面块中的路径对象（简化表示）。
#[derive(Debug, Clone)]
pub struct PageBlockPathObject {
    /// 对象 ID。
    pub id: u32,
    /// 边界框 "x y width height"。
    pub boundary: String,
    /// 缩略路径数据。
    pub abbreviated_data: String,
}

/// 页面块中的图像对象（简化表示）。
#[derive(Debug, Clone)]
pub struct PageBlockImageObject {
    /// 对象 ID。
    pub id: u32,
    /// 边界框 "x y width height"。
    pub boundary: String,
    /// 图像资源引用 ID。
    pub resource_id: u32,
}

impl CT_PageBlock {
    /// 创建空的页面块。
    #[must_use]
    pub fn new() -> Self {
        Self {
            page_blocks: Vec::new(),
            text_objects: Vec::new(),
            path_objects: Vec::new(),
            image_objects: Vec::new(),
        }
    }

    /// 添加嵌套页面块。
    pub fn add_page_block(&mut self, block: CT_PageBlock) {
        self.page_blocks.push(block);
    }

    /// 添加文本对象。
    pub fn add_text_object(&mut self, obj: PageBlockTextObject) {
        self.text_objects.push(obj);
    }

    /// 添加路径对象。
    pub fn add_path_object(&mut self, obj: PageBlockPathObject) {
        self.path_objects.push(obj);
    }

    /// 添加图像对象。
    pub fn add_image_object(&mut self, obj: PageBlockImageObject) {
        self.image_objects.push(obj);
    }

    /// 获取所有嵌套页面块。
    #[must_use]
    pub fn get_page_blocks(&self) -> &[CT_PageBlock] {
        &self.page_blocks
    }

    /// 子元素总数（递归统计）。
    #[must_use]
    pub fn total_count(&self) -> usize {
        let direct = self.text_objects.len() + self.path_objects.len() + self.image_objects.len();
        let nested: usize = self.page_blocks.iter().map(|b| b.total_count()).sum();
        direct + nested
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = String::from("<ofd:PageBlock>\n");
        for text_obj in &self.text_objects {
            let _ = writeln!(
                xml,
                "  <ofd:TextObject ID=\"{}\" Boundary=\"{}\">{}</ofd:TextObject>",
                text_obj.id, text_obj.boundary, text_obj.content
            );
        }
        for path_obj in &self.path_objects {
            let _ = writeln!(
                xml,
                "  <ofd:PathObject ID=\"{}\" Boundary=\"{}\">\
                 <ofd:AbbreviatedData>{}</ofd:AbbreviatedData>\
                 </ofd:PathObject>",
                path_obj.id, path_obj.boundary, path_obj.abbreviated_data
            );
        }
        for img_obj in &self.image_objects {
            let _ = writeln!(
                xml,
                "  <ofd:ImageObject ID=\"{}\" Boundary=\"{}\" ResourceID=\"{}\" />",
                img_obj.id, img_obj.boundary, img_obj.resource_id
            );
        }
        for block in &self.page_blocks {
            xml.push_str(&block.to_xml_string());
        }
        xml.push_str("</ofd:PageBlock>\n");
        xml
    }
}

impl Default for CT_PageBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl PageBlockTextObject {
    /// 创建文本对象。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            content: content.into(),
            font_size: 12.0,
        }
    }

    /// 设置字号。
    #[must_use]
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }
}

impl PageBlockPathObject {
    /// 创建路径对象。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            abbreviated_data: data.into(),
        }
    }
}

impl PageBlockImageObject {
    /// 创建图像对象。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>, resource_id: u32) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            resource_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_page_block_new() {
        let block = CT_PageBlock::new();
        assert!(block.page_blocks.is_empty());
        assert!(block.text_objects.is_empty());
        assert!(block.path_objects.is_empty());
        assert!(block.image_objects.is_empty());
    }

    #[test]
    fn test_ct_page_block_add_text() {
        let mut block = CT_PageBlock::new();
        block.add_text_object(PageBlockTextObject::new(1, "0 0 100 20", "hello"));
        assert_eq!(block.text_objects.len(), 1);
        assert_eq!(block.total_count(), 1);
    }

    #[test]
    fn test_ct_page_block_add_path() {
        let mut block = CT_PageBlock::new();
        block.add_path_object(PageBlockPathObject::new(2, "0 0 50 50", "M0 0L10 10"));
        assert_eq!(block.path_objects.len(), 1);
    }

    #[test]
    fn test_ct_page_block_add_image() {
        let mut block = CT_PageBlock::new();
        block.add_image_object(PageBlockImageObject::new(3, "0 0 100 100", 10));
        assert_eq!(block.image_objects.len(), 1);
    }

    #[test]
    fn test_ct_page_block_nested() {
        let inner = CT_PageBlock::new();
        let mut outer = CT_PageBlock::new();
        outer.add_page_block(inner);
        assert_eq!(outer.get_page_blocks().len(), 1);
    }

    #[test]
    fn test_ct_page_block_total_count_recursive() {
        let mut inner = CT_PageBlock::new();
        inner.add_text_object(PageBlockTextObject::new(1, "0 0 10 10", "x"));
        inner.add_path_object(PageBlockPathObject::new(2, "0 0 10 10", "M0 0"));
        let mut outer = CT_PageBlock::new();
        outer.add_text_object(PageBlockTextObject::new(3, "0 0 10 10", "y"));
        outer.add_page_block(inner);
        assert_eq!(outer.total_count(), 3);
    }

    #[test]
    fn test_ct_page_block_to_xml_basic() {
        let block = CT_PageBlock::new();
        let xml = block.to_xml_string();
        assert!(xml.contains("<ofd:PageBlock>"));
        assert!(xml.contains("</ofd:PageBlock>"));
    }

    #[test]
    fn test_ct_page_block_to_xml_with_objects() {
        let mut block = CT_PageBlock::new();
        block.add_text_object(PageBlockTextObject::new(1, "10 20 50 15", "test").font_size(14.0));
        block.add_image_object(PageBlockImageObject::new(2, "0 0 100 100", 5));
        let xml = block.to_xml_string();
        assert!(xml.contains("ofd:TextObject"));
        assert!(xml.contains("test"));
        assert!(xml.contains("ofd:ImageObject"));
        assert!(xml.contains("ResourceID=\"5\""));
    }

    #[test]
    fn test_ct_page_block_clone_debug() {
        let block = CT_PageBlock::new();
        let block2 = block.clone();
        assert!(block2.text_objects.is_empty());
        assert!(format!("{block:?}").contains("CT_PageBlock"));
    }

    #[test]
    fn test_page_block_text_object_builder() {
        let obj = PageBlockTextObject::new(1, "0 0 50 20", "hello").font_size(18.0);
        assert_eq!(obj.id, 1);
        assert_eq!(obj.content, "hello");
        assert!((obj.font_size - 18.0).abs() < f64::EPSILON);
    }
}
