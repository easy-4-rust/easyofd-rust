//! OFD 文档读取器。

use std::fs::File;
use std::io::{Cursor, Read, Seek};

use easyofd_core::{ContentObject, OfdError, OfdMetadata, OfdPage, OfdResult};
use easyofd_package::validate_archive;

use crate::parser::{
    parse_document_entry, parse_document_resources, parse_ofd_entry, parse_page_entry,
};
use crate::read_options::ReadOptions;

/// OFD 文档读取器。
pub struct OfdReader {
    pages: Vec<OfdPage>,
    metadata: OfdMetadata,
    /// 原始 ZIP 中不由写入器重新生成的条目（模板页、注释、附件、
    /// 签名、自定义标签等容器内容），用于无损 roundtrip。
    raw_entries: Vec<(String, Vec<u8>)>,
}

impl OfdReader {
    /// 从文件路径打开并解析 OFD 文件。
    ///
    /// # 错误
    ///
    /// 文件无法读取或包含无效 OFD 数据时返回错误。
    pub fn open(path: impl AsRef<std::path::Path>) -> OfdResult<Self> {
        Self::open_with_options(path, ReadOptions::default())
    }

    /// 使用指定选项打开 OFD 文件。
    ///
    /// # 错误
    ///
    /// 文件、ZIP 包或 XML 无效时返回错误。
    pub fn open_with_options(
        path: impl AsRef<std::path::Path>,
        options: ReadOptions,
    ) -> OfdResult<Self> {
        let file = File::open(path)?;
        Self::from_seek(file, options)
    }

    /// 从内存字节数组解析 OFD 文件。
    ///
    /// # 错误
    ///
    /// 数据无效时返回错误。
    pub fn from_bytes(data: &[u8]) -> OfdResult<Self> {
        Self::from_seek(Cursor::new(data), ReadOptions::default())
    }

    /// 从实现 `Read + Seek` 的输入读取文档。
    ///
    /// # 错误
    ///
    /// ZIP 包或 XML 无效时返回错误。
    pub fn from_seek<R: Read + Seek>(source: R, options: ReadOptions) -> OfdResult<Self> {
        let mut pages = Vec::new();
        let mut raw_entries = Vec::new();
        let metadata = visit_archive(
            source,
            options,
            |_, page| {
                pages.push(page);
                Ok(())
            },
            &mut raw_entries,
        )?;
        Ok(Self {
            pages,
            metadata,
            raw_entries,
        })
    }

    /// 逐页访问文件，不在内存中保留已经处理过的页面。
    ///
    /// 回调页码从 1 开始。回调返回错误时立即停止解析。
    ///
    /// # 错误
    ///
    /// 文件、ZIP、XML 或页面回调失败时返回错误。
    pub fn visit_path(
        path: impl AsRef<std::path::Path>,
        options: ReadOptions,
        mut visitor: impl FnMut(usize, OfdPage) -> OfdResult<()>,
    ) -> OfdResult<usize> {
        let mut count = 0usize;
        let mut raw_entries = Vec::new();
        visit_archive(
            File::open(path)?,
            options,
            |page_number, page| {
                count += 1;
                visitor(page_number, page)
            },
            &mut raw_entries,
        )?;
        Ok(count)
    }

    /// 文档页数。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// 所有已解析的页面。
    #[must_use]
    pub fn pages(&self) -> &[OfdPage] {
        &self.pages
    }

    /// 文档元数据（从 OFD.xml 提取）。
    #[must_use]
    pub fn metadata(&self) -> &OfdMetadata {
        &self.metadata
    }

    /// 原始 ZIP 中不由写入器重新生成的条目（按名字排序），可用于无损
    /// roundtrip：读取后把这些条目原样写回。
    ///
    /// 排除了 `OFD.xml`、`Document.xml`、`DocumentRes.xml`、`PublicRes.xml`、
    /// 页面内容以及写入器按自身命名规则生成的图片资源。
    #[must_use]
    pub fn raw_entries(&self) -> &[(String, Vec<u8>)] {
        &self.raw_entries
    }

    /// 从所有页面提取文本，每页一个 `String`。
    #[must_use]
    pub fn extract_text(&self) -> Vec<String> {
        self.pages.iter().map(page_text).collect()
    }

    /// 提取所有文本并合并为单个字符串，以页面分隔符分隔。
    #[must_use]
    pub fn extract_all_text(&self) -> String {
        self.extract_text().join("\n---\n")
    }
}

fn visit_archive<R: Read + Seek>(
    source: R,
    options: ReadOptions,
    mut visitor: impl FnMut(usize, OfdPage) -> OfdResult<()>,
    raw_entries: &mut Vec<(String, Vec<u8>)>,
) -> OfdResult<OfdMetadata> {
    let mut archive = zip::ZipArchive::new(source).map_err(|e| OfdError::Zip(e.to_string()))?;
    validate_archive(&mut archive, options.package_limits)?;

    // Collect entries that the writer will not regenerate, so a roundtrip
    // can carry them over verbatim (template pages, annotations, attachments,
    // signatures, custom tags and their payload files).
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| OfdError::Zip(e.to_string()))?;
        let name = file.name().to_string();
        if !writer_regenerates(&name) {
            // Clamp the preallocation hint to a sane bound on 32-bit targets.
            let capacity = usize::try_from(file.size()).unwrap_or(usize::MAX);
            let mut data = Vec::with_capacity(capacity);
            std::io::Read::read_to_end(&mut file, &mut data).map_err(OfdError::Io)?;
            raw_entries.push((name, data));
        }
    }

    let ofd_entry = parse_ofd_entry(&mut archive)?;
    let doc_root = &ofd_entry.doc_root;
    let document_entry = parse_document_entry(&mut archive, doc_root)?;
    let page_refs = &document_entry.pages;
    let resources = parse_document_resources(&mut archive, doc_root)?;
    for (index, page_loc) in page_refs.iter().enumerate() {
        let page_number = index + 1;
        if options.first_page.is_some_and(|first| page_number < first)
            || options.last_page.is_some_and(|last| page_number > last)
        {
            continue;
        }
        let page_path = format!("{doc_root}/{page_loc}");
        let page = parse_page_entry(&mut archive, &page_path, doc_root, &resources)?;
        visitor(page_number, page)?;
    }
    // Parse date strings into NaiveDateTime if present
    let mod_date = ofd_entry.mod_date.as_deref().and_then(|s| {
        // Try ISO format: "2024-05-31" or "2024-05-31T00:00:00"
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
            })
            .ok()
    });
    let creation_date = ofd_entry.creation_date.as_deref().and_then(|s| {
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
            })
            .ok()
    });

    Ok(OfdMetadata {
        doc_id: ofd_entry.doc_id,
        author: ofd_entry.author,
        creator: ofd_entry.creator,
        creator_version: ofd_entry.creator_version,
        mod_date,
        creation_date,
        max_unit_id: ofd_entry.max_unit_id,
        bookmarks: document_entry.bookmarks,
        custom_datas: ofd_entry.custom_datas,
        application_box: document_entry.application_box,
        content_box: document_entry.content_box,
        clip_box: document_entry.clip_box,
        bleed_box: document_entry.bleed_box,
        trim_box: document_entry.trim_box,
        signatures_path: ofd_entry.signatures_path,
        template_pages: document_entry.template_pages,
        annotations_path: document_entry.annotations_path,
        attachments_path: document_entry.attachments_path,
        custom_tags_path: document_entry.custom_tags_path,
        page_area_present: document_entry.page_area_present,
        ..OfdMetadata::default()
    })
}

/// 判断某 ZIP 条目是否由 `OfdWriter` 在写出时重新生成。
///
/// 这些条目在 roundtrip 时不应原样复制（否则会产生重复条目）：
/// 文档主文件、资源清单、页面内容，以及写入器按自身命名规则
/// （`Doc_0/Res/Image_N.*`）生成的图片资源。
fn writer_regenerates(name: &str) -> bool {
    if name == "OFD.xml"
        || name.ends_with("/Document.xml")
        || name.ends_with("/DocumentRes.xml")
        || name.ends_with("/PublicRes.xml")
        // Only page content files are regenerated; directory entries such
        // as "Doc_0/Pages/Page_0/" must be preserved verbatim.
        || (name.contains("/Pages/Page_") && name.ends_with("/Content.xml"))
    {
        return true;
    }
    // Writer-assigned image names: Doc_0/Res/Image_N.<ext>
    if let Some(rest) = name.strip_prefix("Doc_0/Res/Image_") {
        return !rest.is_empty()
            && rest
                .rsplit_once('.')
                .is_some_and(|(idx, _)| idx.chars().all(|c| c.is_ascii_digit()));
    }
    false
}

/// 将页面上所有文本对象合并为一个字符串。
pub(crate) fn page_text(page: &OfdPage) -> String {
    page.content
        .iter()
        .filter_map(|obj| {
            if let ContentObject::Text(t) = obj {
                Some(t.text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
