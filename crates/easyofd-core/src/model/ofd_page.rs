//! OFD 页面定义。

use crate::model::content_object::ContentObject;
use crate::model::image_object::ImageObject;
use crate::model::path_object::PathObject;
use crate::model::text_object::TextObject;

/// OFD 文档中的单个页面。
#[derive(Debug, Clone)]
pub struct OfdPage {
    /// 页面宽度（mm）。
    pub width: f64,
    /// 页面高度（mm）。
    pub height: f64,
    /// 此页面上的内容块。
    pub content: Vec<ContentObject>,
    /// 原始页面路径（相对文档目录，如 `"Pages/Page_Insert_55_2/Content.xml"`）。
    ///
    /// 读取 OFD 文件时保留，写入器优先使用该路径而非自动命名
    /// （`Pages/Page_N/Content.xml`），从而在 roundtrip 时保持页面路径一致。
    pub base_path: Option<String>,
}

impl OfdPage {
    /// 使用给定尺寸创建新页面。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            content: Vec::new(),
            base_path: None,
        }
    }

    /// 设置原始页面路径（roundtrip 保留页面路径时使用）。
    #[must_use]
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// 向此页面添加文本对象。
    pub fn add_text(&mut self, text: TextObject) {
        self.content.push(ContentObject::Text(text));
    }

    /// 向此页面添加图片对象。
    pub fn add_image(&mut self, image: ImageObject) {
        self.content.push(ContentObject::Image(image));
    }

    /// 向此页面添加路径对象。
    pub fn add_path(&mut self, path: PathObject) {
        self.content.push(ContentObject::Path(path));
    }
}
