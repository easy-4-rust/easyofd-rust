//! OFD 文档元数据。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_DocInfo

use chrono::NaiveDateTime;

use crate::model::bookmarks::Bookmarks;
use crate::model::custom_datas::CustomDatas;
use crate::model::permissions::Permissions;
use crate::model::template_page::TemplatePage;

/// OFD 文档元数据（OFD.xml 层级）。
///
/// 对应 Java: ofdrw CT_DocInfo。
// 多个布尔标志反映 GB/T 33190-2016 中同名元素的存在性（ofdrw 对不同场景
// 会省略这些元素），聚合为一个结构体便于 roundtrip 保真。
#[allow(clippy::struct_excessive_bools)]
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
    /// 创建日期原始文本（ofd:CreationDate），roundtrip 保真用。
    ///
    /// 保留 OFD 文件中 `<ofd:CreationDate>` 的原始字符串（如 `"2020-01-25"`），
    /// writer 优先使用此值原样输出，避免强加日期格式（如追加 `T00:00:00`）。
    pub creation_date_raw: Option<String>,
    /// 最后修改日期（ofd:ModDate）。
    pub mod_date: Option<NaiveDateTime>,
    /// 最后修改日期原始文本（ofd:ModDate），roundtrip 保真用。
    ///
    /// 保留 OFD 文件中 `<ofd:ModDate>` 的原始字符串，writer 优先使用此值原样输出。
    pub mod_date_raw: Option<String>,
    /// 最大单元标识符（ofd:MaxUnitID），默认 0。
    pub max_unit_id: u32,
    /// 书签集合（ofd:Bookmarks）。
    pub bookmarks: Option<Bookmarks>,
    /// 大纲集合（ofd:Outlines，即 GB/T 33190-2016 的书签大纲）。
    pub outlines: Option<Bookmarks>,
    /// 自定义数据集合（ofd:CustomDatas）。
    pub custom_datas: Option<CustomDatas>,
    /// 文档用途（ofd:DocUsage），如 "Normal"。
    pub doc_usage: Option<String>,
    /// 文档关键词（ofd:Keywords）。
    pub keywords: Option<String>,
    /// 文档主题（ofd:Subject），对应 ofdrw CT_DocInfo.Subject。
    pub subject: Option<String>,
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
    /// 签名容器路径（ofd:Signatures），位于 OFD.xml 的 DocBody 中，如 "/Doc_0/Signs/Signatures.xml"。
    pub signatures_path: Option<String>,
    /// 模板页引用（ofd:TemplatePage），位于 Document.xml 的 CommonData 中。
    pub template_pages: Vec<TemplatePage>,
    /// 注释容器路径（ofd:Annotations），位于 Document.xml 中，如 "Annots/Annotations.xml"。
    pub annotations_path: Option<String>,
    /// 附件容器路径（ofd:Attachments），位于 Document.xml 中，如 "Attachs/Attachments.xml"。
    pub attachments_path: Option<String>,
    /// 自定义标签容器路径（ofd:CustomTags），位于 Document.xml 中，如 "Tags/CustomTags.xml"。
    pub custom_tags_path: Option<String>,
    /// 原始 Document.xml 是否声明了 ofd:PageArea（ofdrw 未显式设置页面大小时省略该元素）。
    pub page_area_present: bool,
    /// 文档目录（ZIP 中的目录前缀，如 "Doc_0"）。
    pub doc_dir: String,
    /// Document XML 文件名（位于 `doc_dir` 下，通常为 "Document.xml"，
    /// 非标准文件可能是 "Document_0.xml"）。
    pub document_file: String,
    /// DocumentRes 引用（CommonData 中 `<ofd:DocumentRes>` 的文本，如
    /// "DocumentRes.xml" 或非标准的 "DocumentRes_0.xml"）。
    pub document_res: Option<String>,
    /// CommonData 是否声明了 ofd:DocumentRes 元素（ofdrw 样本可能把
    /// PublicRes 引用错误指向 DocumentRes.xml 而无 DocumentRes 元素）。
    pub document_res_element_present: bool,
    /// 源文档是否包含 PublicRes.xml（ofdrw 在无字体资源时不写出该文件，
    /// 但 Document.xml 中的引用仍保留）。
    pub public_res_present: bool,
    /// Document.xml 的 CommonData 是否声明了 ofd:PublicRes 引用。
    pub public_res_element_present: bool,
    /// 文档权限（ofd:Permissions），如存在。
    pub permissions: Option<Permissions>,
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
            creation_date_raw: None,
            mod_date: None,
            mod_date_raw: None,
            max_unit_id: 0,
            bookmarks: None,
            outlines: None,
            custom_datas: None,
            doc_usage: None,
            keywords: None,
            subject: None,
            application_box: None,
            content_box: None,
            clip_box: None,
            bleed_box: None,
            trim_box: None,
            signatures_path: None,
            template_pages: Vec::new(),
            annotations_path: None,
            attachments_path: None,
            custom_tags_path: None,
            page_area_present: true,
            doc_dir: "Doc_0".to_string(),
            document_file: "Document.xml".to_string(),
            document_res: None,
            document_res_element_present: true,
            public_res_present: true,
            public_res_element_present: true,
            permissions: None,
        }
    }
}
