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
}

impl OfdPage {
    /// 使用给定尺寸创建新页面。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            content: Vec::new(),
        }
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
