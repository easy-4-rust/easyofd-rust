//! 页面内容对象枚举。

use crate::model::image_object::ImageObject;
use crate::model::path_object::PathObject;
use crate::model::text_object::TextObject;

/// OFD 页面上的内容对象。
#[derive(Debug, Clone)]
pub enum ContentObject {
    /// 文本块。
    Text(TextObject),
    /// 图片。
    Image(ImageObject),
    /// 矢量路径。
    Path(PathObject),
}
