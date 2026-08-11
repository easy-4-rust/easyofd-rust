//! OFD 文档元数据。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_DocInfo

use chrono::NaiveDateTime;

use crate::model::bookmarks::Bookmarks;
use crate::model::custom_datas::CustomDatas;

/// OFD 文档元数据（OFD.xml 层级）。
///
/// 对应 Java: ofdrw CT_DocInfo。
#[derive(Debug, Clone)]
pub struct OfdMetadata {
    /// 文档版本（默认: "1.0"）。
    pub version: String,
    /// 文档标识符（ofd:DocID）。
    pub doc_id: Option<String>,
    /// 文档标题。
    pub title: Option<String>,
    /// 文档作者（ofd:Author）。
    pub author: Option<String>,
    /// 创建应用程序名称（ofd:Creator）。
    pub creator: Option<String>,
    /// 创建应用程序版本（ofd:CreatorVersion）。
    pub creator_version: Option<String>,
    /// 创建日期（ofd:CreationDate）。
    pub creation_date: Option<NaiveDateTime>,
    /// 最后修改日期（ofd:ModDate）。
    pub mod_date: Option<NaiveDateTime>,
    /// 最大单元标识符（ofd:MaxUnitID），默认 0。
    pub max_unit_id: u32,
    /// 书签集合（ofd:Bookmarks）。
    pub bookmarks: Option<Bookmarks>,
    /// 自定义数据集合（ofd:CustomDatas）。
    pub custom_datas: Option<CustomDatas>,
    /// 应用区域（ofd:ApplicationBox），格式 "x y w h"。
    pub application_box: Option<String>,
    /// 内容区域（ofd:ContentBox），格式 "x y w h"。
    pub content_box: Option<String>,
    /// 裁剪区域（ofd:ClipBox），格式 "x y w h"。
    pub clip_box: Option<String>,
    /// 出血区域（ofd:BleedBox），格式 "x y w h"。
    pub bleed_box: Option<String>,
    /// 裁切区域（ofd:TrimBox），格式 "x y w h"。
    pub trim_box: Option<String>,
}

impl Default for OfdMetadata {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            doc_id: None,
            title: None,
            author: None,
            creator: None,
            creator_version: None,
            creation_date: None,
            mod_date: None,
            max_unit_id: 0,
            bookmarks: None,
            custom_datas: None,
            application_box: None,
            content_box: None,
            clip_box: None,
            bleed_box: None,
            trim_box: None,
        }
    }
}
