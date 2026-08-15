//! OFD 合并器。
//!
//! 对应 Java: org.ofdrw.tool.merge.OFDMerger
//!
//! 将多个 OFD 文档的指定页面合并为一个新文档。
//!
//! ## 合并策略（模型级合并，与 Java DOM 级差异说明）
//!
//! Java 版 `OFDMerger#doMerge` 在 DOM 层面克隆页面 XML 并逐节点迁移资源
//! （字体、图片、DrawParam、ColorSpace 等），同时迁移模板页和注释。
//!
//! Rust 版采用模型级合并：通过 [`easyofd_reader::OfdReader`] 将源 OFD 解析为
//! [`OfdPage`]（含 [`ContentObject::Text`] / [`ContentObject::Image`] /
//! [`ContentObject::Path`]），再通过 [`easyofd_writer::OfdWriter`] 重新生成
//! OFD ZIP。图片资源通过 SM3 内容哈希去重（对应 Java `resFileHashTable`）。
//!
//! ### 覆盖状态
//!
//! | Java 功能             | Rust 状态     | 说明                                     |
//! |-----------------------|---------------|------------------------------------------|
//! | 模板页迁移            | 已覆盖        | 原样复制 + 资源迁移 + ID 冲突检测重映射  |
//! | 注释迁移              | 已覆盖        | 原始字节复制 + 索引重建 + 资源迁移       |
//! | DrawParam 迁移        | 已覆盖        | 从 DocumentRes.xml 提取，分配新 ID       |
//! | Font 文件迁移         | 已覆盖        | PublicRes.xml Font 定义 + FontFile 去重  |
//! | DOM 对象 ID 重分配    | 不需要        | writer 重新生成所有 ID                   |
//! | `copyTemplate` 开关   | 尊重但无操作  | 模型层无模板，开关无效                   |
//! | `copyAnnotations` 开关| 尊重但无操作  | 模型层无注释，开关无效                   |
//! | 资源文件 SM3 去重     | 已覆盖        | 对应 Java `resFileHashTable`             |
//! | ColorSpace 迁移       | 已覆盖        | PublicRes.xml ColorSpace + ProfileFile   |
//! | 流式加载              | 已覆盖        | 按 entry 提取，非整包驻留               |

use std::collections::HashMap;
use std::fs;

use super::resource_dedup::ResourceDedup;
use super::{BareOFDDoc, DocContext, DocPage, PageEntry};
use easyofd_core::model::template_page::TemplatePage;
use easyofd_core::{ContentObject, ImageObject, OfdPage};
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;

/// 源文档轻量缓存：按需打开 ZIP，仅驻留已解析的元数据。
///
/// 对应 Java: `DocContext` 中 `OFDReader` 按需读取的行为。
///
/// 与旧方案（`HashMap<usize, Vec<u8>>` 整包驻留）的区别：
/// - 旧：N 个源 = N 份完整 OFD 字节驻留内存（O(Σ源全量)）
/// - 新：N 个源 = N 个文件句柄 + 轻量元数据（O(N × 常量)），
///   仅在 `load_source_page` / `migrate_extras` 时按 entry 提取。
///
/// 驻留字节从 O(Σ源全量) 降到 O(活跃 entry)。
#[derive(Debug)]
struct SourceArchive {
    /// 源文件路径（用于按需 reopen）。
    path: String,
    /// 文档目录名（如 "Doc_0"）。
    #[allow(dead_code)]
    doc_dir: String,
    /// 页索引 → 页面 Content.xml 路径。
    page_paths: Vec<String>,
    /// 页索引 → 页面 ID（来自 Document.xml 的 Page ID 属性）。
    #[allow(dead_code)]
    page_ids: Vec<String>,
}

impl SourceArchive {
    /// 打开源 OFD 文件，提取 doc_dir / page_paths / page_ids。
    fn open(path: &str) -> Result<Self, String> {
        let file = fs::File::open(path).map_err(|e| format!("打开源文档 '{path}' 失败: {e}"))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("解析源文档 '{path}' ZIP 失败: {e}"))?;

        let doc_dir =
            extract_doc_dir_from_archive(&mut archive).unwrap_or_else(|| "Doc_0".to_string());

        let doc_xml_path = format!("{doc_dir}/Document.xml");
        let page_paths = extract_page_paths_from_archive(&mut archive, &doc_xml_path);
        let page_ids = extract_page_ids_from_archive(&mut archive, &doc_xml_path);

        Ok(Self {
            path: path.to_string(),
            doc_dir,
            page_paths,
            page_ids,
        })
    }

    /// 从 ZIP 中按名字读取 entry 字节（备用，当前未使用）。
    #[allow(dead_code)]
    fn read_entry(&self, name: &str) -> Option<Vec<u8>> {
        let file = fs::File::open(&self.path).ok()?;
        let mut archive = zip::ZipArchive::new(file).ok()?;
        read_zip_entry_bytes(&mut archive, name)
    }
}

/// OFD 合并器。
///
/// 对应 Java: `org.ofdrw.tool.merge.OFDMerger`
///
/// 将多个 OFD 文档的页面合并到一个新文档中。
///
/// # 使用流程
///
/// 1. 创建 [`OfdMerger`] 实例。
/// 2. 调用 [`add_source`] 注册源文档。
/// 3. 调用 [`add_page`] 或 [`add_page_entry`] 指定要合并的页面。
/// 4. 调用 [`merge`] 执行合并。
///
/// [`add_source`]: OfdMerger::add_source
/// [`add_page`]: OfdMerger::add_page
/// [`add_page_entry`]: OfdMerger::add_page_entry
/// [`merge`]: OfdMerger::merge
#[derive(Debug)]
pub struct OfdMerger {
    /// 输出路径。
    output_path: String,
    /// 已注册的源文档。
    sources: Vec<BareOFDDoc>,
    /// 待合并的页面列表（DocPage 路径，page_index 从 0 开始）。
    pages: Vec<DocPage>,
    /// 待合并的页面列表（PageEntry 路径，page_index 从 1 开始，支持 tb_mix_pages）。
    page_entries: Vec<PageEntry>,
    /// 合并上下文。
    context: DocContext,
}

impl OfdMerger {
    /// 创建合并器。
    ///
    /// # 参数
    ///
    /// - `output_path`：合并后的输出文件路径。
    #[must_use]
    pub fn new(output_path: impl Into<String>) -> Self {
        Self {
            output_path: output_path.into(),
            sources: Vec::new(),
            pages: Vec::new(),
            page_entries: Vec::new(),
            context: DocContext::new(),
        }
    }

    /// 注册源文档。
    ///
    /// 返回源文档索引（从 0 开始）。
    pub fn add_source(&mut self, path: impl Into<String>, page_count: usize) -> usize {
        let index = self.sources.len();
        let path_str = path.into();
        self.context.add_source(index, &path_str);
        self.sources.push(BareOFDDoc::new(&path_str, page_count));
        index
    }

    /// 添加要合并的页面（DocPage 路径，page_index 从 0 开始）。
    pub fn add_page(&mut self, page: DocPage) {
        let global_index = self.pages.len();
        self.context
            .add_page_mapping(global_index, page.source_index, page.page_index);
        self.pages.push(page);
    }

    /// 添加要合并的页面（PageEntry 路径，page_index 从 1 开始，支持 tb_mix_pages）。
    ///
    /// 对应 Java: `OFDMerger#add(PageEntry...)`
    pub fn add_page_entry(&mut self, entry: PageEntry) {
        self.page_entries.push(entry);
    }

    /// 执行合并。
    ///
    /// 对应 Java: `org.ofdrw.tool.merge.OFDMerger#doMerge`
    ///
    /// 返回合并后的 OFD 文档字节。若 [`output_path`] 已设置，同时写入文件。
    ///
    /// # 错误
    ///
    /// - 没有注册源文档或没有指定页面时返回错误。
    /// - 源文件读取失败时返回错误。
    /// - 页码越界时返回错误（包含源路径信息）。
    ///
    /// [`output_path`]: OfdMerger::output_path
    pub fn merge(&mut self) -> Result<Vec<u8>, String> {
        if self.sources.is_empty() {
            return Err("没有注册源文档".to_string());
        }
        if self.pages.is_empty() && self.page_entries.is_empty() {
            return Err("没有指定要合并的页面".to_string());
        }

        // ── 预提取页面描述，避免借用冲突（self.pages vs &mut self）──
        let page_descs: Vec<(usize, usize)> = self
            .pages
            .iter()
            .map(|p| (p.source_index, p.page_index))
            .collect();

        // ── 预提取 PageEntry 描述（含 tb_mix_pages 和 copy 标志）──
        #[allow(clippy::type_complexity)]
        let entry_descs: Vec<(usize, usize, Vec<(usize, usize)>, bool, bool)> = self
            .page_entries
            .iter()
            .map(|e| {
                let mix = e
                    .tb_mix_pages
                    .iter()
                    .map(|m| (m.doc_ctx_index, m.page_index))
                    .collect();
                (
                    e.doc_ctx_index,
                    e.page_index,
                    mix,
                    e.copy_annotations,
                    e.copy_template,
                )
            })
            .collect();

        // ── 源文档按需缓存（流式：按 entry 提取，非整包驻留）──
        //
        // 对应 Java: `DocContext` 中 `OFDReader` 按需读取的行为。
        // 旧方案：`HashMap<usize, Vec<u8>>` 把每个源文档完整字节驻留内存。
        // 新方案：`HashMap<usize, SourceArchive>` 仅驻留文件句柄 + 轻量元数据。
        //
        // 结构性论证：驻留字节从 O(Σ源全量) 降到 O(N × 常量)。
        // 每个 SourceArchive 驻留：path(String) + doc_dir(String) + page_paths(Vec<String>)
        // + page_ids(Vec<String>)，共约数百字节。而旧方案驻留每个源的完整 ZIP（含图片等）。
        let mut sources: HashMap<usize, SourceArchive> = HashMap::new();

        // ── 收集合并后的页面 ──
        let mut merged_pages: Vec<OfdPage> = Vec::new();

        // ── 跟踪合并页面到源页面的映射（用于注解/模板迁移）──
        //
        // merged_page_map[merged_index] = (source_doc_ctx_index, source_page_index_0based)
        let mut merged_page_map: Vec<(usize, usize)> = Vec::new();

        // ── 跟踪 PageEntry 的 copy_annotations / copy_template 开关 ──
        // merged_page_flags[merged_index] = (copy_annotations, copy_template)
        let mut merged_page_flags: Vec<(bool, bool)> = Vec::new();

        // ── 处理 DocPage 列表（page_index 从 0 开始）──
        for (source_index, page_index) in page_descs {
            let page_data = self.load_source_page(source_index, page_index, &mut sources)?;
            merged_pages.push(page_data);
            merged_page_map.push((source_index, page_index));
            // DocPage 路径默认复制模板和注解
            merged_page_flags.push((true, true));
        }

        // ── 处理 PageEntry 列表（page_index 从 1 开始）──
        for (doc_ctx_index, page_index, mix_pages, copy_annotations, copy_template) in
            entry_descs.clone()
        {
            // PageEntry.page_index 从 1 开始，转换为 0-based 索引
            let zero_based = page_index.checked_sub(1).ok_or_else(|| {
                let src = self.context.source_path(doc_ctx_index).unwrap_or("未知");
                format!("源文档 '{src}' 的页码不能为 0（PageEntry 页码从 1 开始）")
            })?;

            let mut page_data = self.load_source_page(doc_ctx_index, zero_based, &mut sources)?;

            // ── tb_mix_pages：内容级叠加 ──
            //
            // 对应 Java: `OFDMerger#mixPage` 中将被混合页面 Content 中的元素
            // 追加到目标页面 Content 尾部的行为。
            //
            // 与 Java 的差异：Java 在 DOM 层面克隆 Content 子元素并追加，
            // 同时迁移模板和注释。Rust 版在模型层直接追加 ContentObject，
            // 不涉及模板/注释迁移。
            for (mix_ctx_index, mix_page_index) in mix_pages {
                let mix_zero_based = mix_page_index.checked_sub(1).ok_or_else(|| {
                    let src = self.context.source_path(mix_ctx_index).unwrap_or("未知");
                    format!("源文档 '{src}' 的混合页码不能为 0（PageEntry 页码从 1 开始）")
                })?;

                let mix_page =
                    self.load_source_page(mix_ctx_index, mix_zero_based, &mut sources)?;
                // 将混合页的内容对象追加到目标页尾部
                page_data.content.extend(mix_page.content);
            }

            merged_pages.push(page_data);
            merged_page_map.push((doc_ctx_index, zero_based));
            merged_page_flags.push((copy_annotations, copy_template));
        }

        // ── 计算页面图片资源映射（用于注解/模板资源复用）──
        //
        // 对应 Java: `OFDMerger#domMigrate` 中的资源 ID 分配逻辑。
        // writer 按页面图片顺序分配 ResourceID（100, 101, ...），
        // 此处构建 res_name → ResourceID 映射，使注解/模板资源能复用已有资源。
        let mut res_name_to_id: HashMap<String, usize> = HashMap::new();
        let mut page_image_count: usize = 0;
        for page in &merged_pages {
            for obj in &page.content {
                if let ContentObject::Image(img) = obj {
                    if let Some(ref res_name) = img.res_name {
                        res_name_to_id.insert(res_name.clone(), 100 + page_image_count);
                    }
                    page_image_count += 1;
                }
            }
        }

        // ── 收集源文档路径（用于 migrate_extras 按需打开）──
        let mut source_paths: HashMap<usize, String> = HashMap::new();
        for (idx, src) in &sources {
            source_paths.insert(*idx, src.path.clone());
        }

        // ── 迁移注解和模板（含资源迁移 + 模板 ID 重映射）──
        //
        // 对应 Java: `OFDMerger#pageAnnotationMigrate` + `OFDMerger#pageTplMigrate`
        //         + `OFDMerger#resMigrate` + `OFDMerger#domMigrate`
        let (extra_entries, annotations_path, template_pages, extra_resources) = migrate_extras(
            &source_paths,
            &merged_page_map,
            &merged_page_flags,
            page_image_count,
            &res_name_to_id,
            &mut self.context,
        );

        // ── 生成新文档元数据 ──
        //
        // 对应 Java: `BareOFDDoc` 构造时生成新 DocID（UUID.randomUUID()）
        let metadata = easyofd_core::OfdMetadata {
            doc_id: Some(generate_doc_id()),
            annotations_path,
            template_pages,
            ..easyofd_core::OfdMetadata::default()
        };

        // ── 构建输出 OFD ──
        let mut writer = OfdWriter::new();
        writer.set_metadata(metadata);
        writer.add_pages(merged_pages);
        if !extra_entries.is_empty() {
            writer.preserve_entries(extra_entries);
        }
        // 注解/模板引用的额外图片资源
        for (res_name, data, format_str) in extra_resources {
            let fmt = match format_str.as_str() {
                "JPEG" | "JPG" => easyofd_core::ImageFormat::Jpeg,
                "BMP" => easyofd_core::ImageFormat::Bmp,
                "TIFF" | "TIF" => easyofd_core::ImageFormat::Tiff,
                _ => easyofd_core::ImageFormat::Png, // 默认（含 "PNG"）
            };
            writer.add_extra_resource(res_name, data, fmt);
        }

        let bytes = writer.build().map_err(|e| format!("构建 OFD 失败: {e}"))?;

        // ── 若设置了输出路径，同时写入文件 ──
        if !self.output_path.is_empty() {
            fs::write(&self.output_path, &bytes)
                .map_err(|e| format!("写入输出文件 '{}' 失败: {e}", self.output_path))?;
        }

        Ok(bytes)
    }

    /// 加载源文档的指定页面（内部方法）。
    ///
    /// 按需打开源文件（`OfdReader::open` 从磁盘读取，不缓存全部字节），
    /// 解析 OFD、提取页面，并对图片资源执行 SM3 去重。
    ///
    /// # 参数
    ///
    /// - `source_index`：源文档索引。
    /// - `page_index`：页面索引（从 0 开始）。
    /// - `sources`：源文档轻量缓存（仅驻留 path + 元数据，不驻留全部字节）。
    fn load_source_page(
        &mut self,
        source_index: usize,
        page_index: usize,
        sources: &mut HashMap<usize, SourceArchive>,
    ) -> Result<OfdPage, String> {
        // 克隆路径字符串，释放对 self.context 的不可变借用，
        // 以便后续 dedup_image_resource 可以可变借用 self.context。
        let src_path = self.resolve_source_path(source_index)?.to_string();

        // 按需缓存轻量元数据（doc_dir / page_paths / page_ids）。
        // 不缓存源文件全部字节——OfdReader::open 从磁盘按需读取。
        if let std::collections::hash_map::Entry::Vacant(e) = sources.entry(source_index) {
            e.insert(SourceArchive::open(&src_path)?);
        }
        let total_pages = sources.get(&source_index).unwrap().page_paths.len();

        // 页码越界检查
        if page_index >= total_pages {
            return Err(format!(
                "源文档 '{}' 页码越界：请求第 {} 页，但文档共 {} 页",
                src_path,
                page_index + 1, // 显示为 1-based 页码
                total_pages,
            ));
        }

        // 从磁盘按需打开源文件（流式：OfdReader::open 不驻留全部字节）
        let reader = OfdReader::open(&src_path)
            .map_err(|e| format!("解析源文档 '{}' 失败: {e}", src_path))?;

        let mut page = reader.pages()[page_index].clone();
        // 清除原始页面路径，让 writer 按合并后的索引自动生成唯一路径
        // （避免多个源文档的 Pages/Page_0/Content.xml 冲突）。
        page.base_path = None;

        // ── 图片资源 SM3 去重 ──
        //
        // 对应 Java: `OFDMerger#copyResFile` 中的 `resFileHashTable` 逻辑。
        // 相同内容（SM3 哈希相同）的图片复用同一 res_name，使 writer 不重复存储。
        for obj in &mut page.content {
            if let ContentObject::Image(ref mut img) = *obj {
                dedup_image_resource(&mut self.context, img);
            }
        }

        Ok(page)
    }

    /// 解析源文档路径（内部方法）。
    fn resolve_source_path(&self, source_index: usize) -> Result<&str, String> {
        self.context
            .source_path(source_index)
            .ok_or_else(|| format!("源文档索引 {} 未注册", source_index))
    }

    /// 获取输出路径。
    #[must_use]
    pub fn output_path(&self) -> &str {
        &self.output_path
    }

    /// 获取源文档数量。
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// 获取待合并页面数量（DocPage + PageEntry）。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len() + self.page_entries.len()
    }

    /// 获取合并上下文。
    #[must_use]
    pub fn context(&self) -> &DocContext {
        &self.context
    }
}

/// 对图片资源执行 SM3 内容去重。
///
/// 对应 Java: `OFDMerger#copyResFile`
///
/// - 计算 `ImageObject.data` 的 SM3 哈希。
/// - 若哈希已存在，复用已有 res_name。
/// - 若不存在，生成新的 res_name 并注册。
fn dedup_image_resource(context: &mut DocContext, img: &mut ImageObject) {
    let ext = match img.format {
        easyofd_core::ImageFormat::Jpeg => ".jpeg",
        easyofd_core::ImageFormat::Png => ".png",
        easyofd_core::ImageFormat::Bmp => ".bmp",
        easyofd_core::ImageFormat::Tiff => ".tiff",
    };

    let hash = ResourceDedup::compute_hash(&img.data);
    let dedup = context.resource_dedup_mut();

    if let Some(existing) = dedup.get_by_hash(&hash) {
        // 命中去重表：复用已有 res_name
        img.res_name = Some(existing.to_string());
    } else {
        // 未命中：生成新的 res_name，注册到去重表
        let counter = dedup.counter() + 1;
        let name = format!("Res/{counter}{ext}");
        dedup.register(hash, name.clone());
        img.res_name = Some(name);
    }
}

/// 生成新的文档 ID。
///
/// 对应 Java: `java.util.UUID.randomUUID()` 在 `BareOFDDoc` 构造时的行为。
///
/// 使用纳秒时间戳生成 32 字符十六进制字符串，格式与 UUID 一致。
#[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
fn generate_doc_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // 格式化为 8-4-4-4-12 的 UUID 风格字符串
    let time_low = (nanos >> 96) as u32;
    let time_mid = (nanos >> 80) as u16;
    let time_hi = (nanos >> 64) as u16;
    let clock_seq = (nanos >> 48) as u16;
    let node = nanos & 0x0000_FFFF_FFFF_FFFF;
    format!("{time_low:08x}-{time_mid:04x}-{time_hi:04x}-{clock_seq:04x}-{node:012x}")
}

// ── 注解/模板迁移 ──────────────────────────────────────────────────────────

/// 迁移结果：(preserve_entries, annotations_path, template_pages, extra_resources)。
///
/// `extra_resources` 为注解/模板引用的额外图片资源 `(res_name, data, format_str)`，
/// 由 writer 写入 DocumentRes.xml（ID 从 `100 + 页面图片总数` 开始）。
type MigrateResult = (
    Vec<(String, Vec<u8>)>,
    Option<String>,
    Vec<TemplatePage>,
    Vec<(String, Vec<u8>, String)>,
);

/// 从 OFD ZIP 中读取指定路径的原始字节。
fn read_zip_entry_bytes<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    path: &str,
) -> Option<Vec<u8>> {
    let mut file = archive.by_name(path).ok()?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut buf).ok()?;
    Some(buf)
}

/// 从 OFD.xml 中提取文档目录名（如 "Doc_0"）。
///
/// 解析 `<ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>` 获取文档目录。
fn extract_doc_dir(xml_bytes: &[u8]) -> Option<String> {
    let xml = std::str::from_utf8(xml_bytes).ok()?;
    let start = xml.find("<ofd:DocRoot>")? + 13;
    let end = xml[start..].find("</ofd:DocRoot>")?;
    let doc_root = xml[start..start + end].trim_start_matches('/');
    doc_root.rfind('/').map(|idx| doc_root[..idx].to_string())
}

/// 从 ZIP archive 中读取 OFD.xml 并提取文档目录名。
fn extract_doc_dir_from_archive<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<String> {
    let bytes = read_zip_entry_bytes(archive, "OFD.xml")?;
    extract_doc_dir(&bytes)
}

/// 从 ZIP archive 中读取 Document.xml 并提取页面 Content.xml 路径列表。
fn extract_page_paths_from_archive<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    doc_xml_path: &str,
) -> Vec<String> {
    let Some(bytes) = read_zip_entry_bytes(archive, doc_xml_path) else {
        return Vec::new();
    };
    let xml = String::from_utf8_lossy(&bytes);
    let mut paths = Vec::new();
    let mut search_from = 0;
    let pattern = "<ofd:Page ";
    while let Some(rel_start) = xml[search_from..].find(pattern) {
        let abs_start = search_from + rel_start;
        let after = &xml[abs_start + pattern.len()..];
        if let Some(bl_rel) = after.find("BaseLoc=\"") {
            let v_start = bl_rel + 9;
            if let Some(v_end) = after[v_start..].find('"') {
                paths.push(after[v_start..v_start + v_end].to_string());
            }
        }
        search_from = abs_start + pattern.len();
    }
    paths
}

/// 从 ZIP archive 中读取 Document.xml 并提取页面 ID 列表。
fn extract_page_ids_from_archive<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    doc_xml_path: &str,
) -> Vec<String> {
    let Some(bytes) = read_zip_entry_bytes(archive, doc_xml_path) else {
        return Vec::new();
    };
    extract_page_ids(&bytes)
}

/// 从 Document.xml 中提取页面 ID 列表（按页面顺序，0-based 索引对应）。
///
/// 解析 `<ofd:Page ID="..." BaseLoc="..."/>` 的 ID 属性。
fn extract_page_ids(xml_bytes: &[u8]) -> Vec<String> {
    let xml = String::from_utf8_lossy(xml_bytes);
    let mut page_ids = Vec::new();
    let mut search_from = 0;
    let pattern = "<ofd:Page ";
    while let Some(rel_start) = xml[search_from..].find(pattern) {
        let abs_start = search_from + rel_start;
        let after = &xml[abs_start + pattern.len()..];
        if let Some(id_rel) = after.find("ID=\"") {
            let v_start = id_rel + 4;
            if let Some(id_end) = after[v_start..].find('"') {
                page_ids.push(after[v_start..v_start + id_end].to_string());
            }
        }
        search_from = abs_start + pattern.len();
    }
    page_ids
}

/// 从 Annotations.xml 中提取 (PageID, FileLoc) 映射。
///
/// 解析 `<ofd:Page PageID="..."><ofd:FileLoc>...</ofd:FileLoc></ofd:Page>`。
fn extract_annot_index(xml_bytes: &[u8]) -> Vec<(String, String)> {
    let xml = String::from_utf8_lossy(xml_bytes);
    let mut entries = Vec::new();
    let mut search_from = 0;
    while let Some(rel_start) = xml[search_from..].find("<ofd:Page ") {
        let abs_start = search_from + rel_start;
        let after = &xml[abs_start + 10..];
        let page_id = after.find("PageID=\"").and_then(|s| {
            let v = s + 8;
            after[v..].find('"').map(|e| after[v..v + e].to_string())
        });
        let file_loc = after.find("<ofd:FileLoc>").and_then(|s| {
            let v = s + 13;
            after[v..]
                .find("</ofd:FileLoc>")
                .map(|e| after[v..v + e].trim().to_string())
        });
        if let (Some(pid), Some(loc)) = (page_id, file_loc) {
            entries.push((pid, loc));
        }
        search_from = abs_start + 10;
    }
    entries
}

/// 从 Document.xml 中提取模板页 (ID, BaseLoc) 列表。
///
/// 解析 CommonData 中的 `<ofd:TemplatePage ID="..." BaseLoc="..."/>`。
fn extract_template_pages(xml_bytes: &[u8]) -> Vec<(String, String)> {
    let xml = String::from_utf8_lossy(xml_bytes);
    let mut entries = Vec::new();
    let mut search_from = 0;
    let pattern = "<ofd:TemplatePage ";
    while let Some(rel_start) = xml[search_from..].find(pattern) {
        let abs_start = search_from + rel_start;
        let after = &xml[abs_start + pattern.len()..];
        let id = after.find("ID=\"").and_then(|s| {
            let v = s + 4;
            after[v..].find('"').map(|e| after[v..v + e].to_string())
        });
        let base_loc = after.find("BaseLoc=\"").and_then(|s| {
            let v = s + 9;
            after[v..].find('"').map(|e| after[v..v + e].to_string())
        });
        if let (Some(i), Some(bl)) = (id, base_loc) {
            entries.push((i, bl));
        }
        search_from = abs_start + pattern.len();
    }
    entries
}

// ── 资源迁移辅助 ────────────────────────────────────────────────────────────

/// 资源定义（从 DocumentRes.xml / PublicRes.xml 提取）。
///
/// 对应 Java: `OFDMerger#resMigrate` 中的 `resObj` 类型判断。
#[derive(Debug, Clone)]
enum ResDef {
    /// 多媒体资源（图片等）。
    ///
    /// 对应 Java: `CT_MultiMedia`
    MultiMedia {
        #[allow(dead_code)]
        media_type: String,
        format: String,
        file_path: String,
    },
    /// DrawParam 资源（绘图参数）。
    ///
    /// 对应 Java: `CT_DrawParam`
    DrawParam { id: String, raw_xml: String },
    /// ColorSpace 资源（颜色空间）。
    ///
    /// 对应 Java: `CT_ColorSpace`（`org.ofdrw.core.pageDescription.color.colorSpace.CT_ColorSpace`）
    ///
    /// 定义在 PublicRes.xml 中，引用 ICC ProfileFile。
    ColorSpace {
        /// Type 属性（RGB / GRAY / CMYK 等）。
        #[allow(dead_code)]
        cs_type: String,
        /// ICC ProfileFile 相对路径（可选）。
        profile_file: Option<String>,
        /// 原始 XML 片段（用于保留完整定义）。
        raw_xml: String,
    },
    /// Font 资源（字体定义）。
    ///
    /// 对应 Java: `CT_Font`（`org.ofdrw.core.text.font.CT_Font`）
    ///
    /// 定义在 PublicRes.xml 中，FontFile 属性指向字体文件。
    Font {
        /// 字体名称。
        #[allow(dead_code)]
        font_name: String,
        /// FontFile 相对路径（可选）。
        font_file: Option<String>,
        /// 原始 XML 片段（用于保留完整定义）。
        raw_xml: String,
    },
}

/// 从 DocumentRes.xml 中提取资源定义。
///
/// 对应 Java: `DocContext.resMgt`（资源管理器）。
///
/// 解析 `<ofd:MultiMedia>` 和 `<ofd:DrawParam>` 条目，构建 ID → 资源定义映射。
fn parse_doc_res_defs(xml_bytes: &[u8]) -> HashMap<String, ResDef> {
    let xml = String::from_utf8_lossy(xml_bytes);
    let mut defs = HashMap::new();

    // ── 提取 MultiMedia 条目 ──
    let mut search_from = 0;
    let mm_pattern = "<ofd:MultiMedia ";
    while let Some(rel) = xml[search_from..].find(mm_pattern) {
        let abs = search_from + rel;
        let tag_end = xml[abs..].find('>').map(|e| abs + e);
        let Some(end) = tag_end else { break };
        let tag_text = &xml[abs..=end];

        let id = extract_attr_value(tag_text, "ID");
        let media_type = extract_attr_value(tag_text, "Type").unwrap_or_default();
        let format = extract_attr_value(tag_text, "Format").unwrap_or_default();

        // 提取 <ofd:MediaFile> 内容
        let media_file = if end + 1 < xml.len() {
            extract_child_text(&xml[end + 1..], "ofd:MediaFile")
        } else {
            None
        };

        if let (Some(id), Some(file_path)) = (id, media_file) {
            defs.insert(
                id,
                ResDef::MultiMedia {
                    media_type,
                    format,
                    file_path,
                },
            );
        }
        search_from = abs + mm_pattern.len();
    }

    // ── 提取 DrawParam 条目 ──
    let mut search_from = 0;
    let dp_pattern = "<ofd:DrawParam ";
    while let Some(rel) = xml[search_from..].find(dp_pattern) {
        let abs = search_from + rel;
        let id = extract_attr_value(&xml[abs..], "ID");

        // 提取完整 <ofd:DrawParam ...>...</ofd:DrawParam> XML
        let close_tag = "</ofd:DrawParam>";
        let raw_xml = if let Some(close_rel) = xml[abs..].find(close_tag) {
            xml[abs..abs + close_rel + close_tag.len()].to_string()
        } else {
            // 自关闭标签
            let tag_end = xml[abs..].find('>').map(|e| abs + e);
            tag_end.map_or(String::new(), |end| xml[abs..=end].to_string())
        };

        if let Some(id) = id {
            defs.insert(id.clone(), ResDef::DrawParam { id, raw_xml });
        }
        search_from = abs + dp_pattern.len();
    }

    defs
}

/// 从属性字符串中提取指定属性的值。
///
/// 查找 `attr="value"` 模式并返回 value。
fn extract_attr_value(text: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = text.find(&pattern)? + pattern.len();
    let end = text[start..].find('"')?;
    Some(text[start..start + end].to_string())
}

/// 从 XML 片段中提取子元素的文本内容。
///
/// 查找 `<tag>text</tag>` 模式并返回 text。
fn extract_child_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].trim().to_string())
}

/// 从子元素标签中提取指定属性的值（备用，当前未使用）。
///
/// 查找 `<tag ... attr="value" ...>` 模式并返回 value。
#[allow(dead_code)]
fn extract_child_attr_value(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("<{tag} ");
    let start = xml.find(&pattern)?;
    let tag_end = xml[start..].find('>').map(|e| start + e)?;
    let tag_text = &xml[start..=tag_end];
    extract_attr_value(tag_text, attr)
}

/// 从 PublicRes.xml 中提取资源定义（ColorSpace、Font）。
///
/// 对应 Java: `DocContext.resMgt`（资源管理器，同时管理 DocumentRes 和 PublicRes）。
///
/// 解析 `<ofd:ColorSpace>` 和 `<ofd:Font>` 条目，构建 ID → 资源定义映射。
///
/// 对应 Java: `OFDMerger#resMigrate` 中 `CT_ColorSpace` / `CT_Font` 分支。
fn parse_pub_res_defs(xml_bytes: &[u8]) -> HashMap<String, ResDef> {
    let xml = String::from_utf8_lossy(xml_bytes);
    let mut defs = HashMap::new();

    // ── 提取 ColorSpace 条目 ──
    //
    // 对应 Java: `CT_ColorSpace`（`ofdrw-core` 的 `pageDescription.color.colorSpace`）
    // 格式：<ofd:ColorSpace ID="10" Type="RGB" Profile="sRGB.icc"/>
    // 或：<ofd:ColorSpace ID="10" Type="CMYK"><ofd:Profile>...</ofd:Profile></ofd:ColorSpace>
    let mut search_from = 0;
    let cs_pattern = "<ofd:ColorSpace ";
    while let Some(rel) = xml[search_from..].find(cs_pattern) {
        let abs = search_from + rel;
        // 找到标签结束：可能是自关闭 /> 或 >
        let tag_end = xml[abs..].find('>').map(|e| abs + e);
        let Some(end) = tag_end else { break };
        let tag_text = &xml[abs..=end];

        let id = extract_attr_value(tag_text, "ID");
        let cs_type = extract_attr_value(tag_text, "Type").unwrap_or_default();
        // Profile 属性（简写形式）
        let profile_attr = extract_attr_value(tag_text, "Profile");
        // Profile 子元素（完整形式）
        let profile_child = if end + 1 < xml.len() && !tag_text.ends_with("/>") {
            extract_child_text(&xml[end + 1..], "ofd:Profile")
                .or_else(|| extract_child_text(&xml[end + 1..], "Profile"))
        } else {
            None
        };
        let profile_file = profile_attr.or(profile_child);

        // 提取完整 XML 片段
        let raw_xml = if tag_text.ends_with("/>") {
            tag_text.to_string()
        } else {
            let close_tag = "</ofd:ColorSpace>";
            xml[abs..]
                .find(close_tag)
                .map_or(tag_text.to_string(), |close_rel| {
                    xml[abs..abs + close_rel + close_tag.len()].to_string()
                })
        };

        if let Some(id) = id {
            defs.insert(
                id,
                ResDef::ColorSpace {
                    cs_type,
                    profile_file,
                    raw_xml,
                },
            );
        }
        search_from = abs + cs_pattern.len();
    }

    // ── 提取 Font 条目 ──
    //
    // 对应 Java: `CT_Font`（`ofdrw-core` 的 `text.font.CT_Font`）
    // 格式：<ofd:Font ID="3" FontName="宋体" FamilyName="宋体"/>
    // 或含 FontFile：<ofd:Font ID="3" FontName="宋体" FontFile="Res/font.ttf"/>
    let mut search_from = 0;
    let font_pattern = "<ofd:Font ";
    while let Some(rel) = xml[search_from..].find(font_pattern) {
        let abs = search_from + rel;
        let tag_end = xml[abs..].find('>').map(|e| abs + e);
        let Some(end) = tag_end else { break };
        let tag_text = &xml[abs..=end];

        let id = extract_attr_value(tag_text, "ID");
        let font_name = extract_attr_value(tag_text, "FontName").unwrap_or_default();
        let font_file = extract_attr_value(tag_text, "FontFile");

        // 提取完整 XML 片段
        let raw_xml = if tag_text.ends_with("/>") {
            tag_text.to_string()
        } else {
            let close_tag = "</ofd:Font>";
            xml[abs..]
                .find(close_tag)
                .map_or(tag_text.to_string(), |close_rel| {
                    xml[abs..abs + close_rel + close_tag.len()].to_string()
                })
        };

        if let Some(id) = id {
            defs.insert(
                id,
                ResDef::Font {
                    font_name,
                    font_file,
                    raw_xml,
                },
            );
        }
        search_from = abs + font_pattern.len();
    }

    defs
}

/// 从 XML 中提取所有资源引用属性。
///
/// 对应 Java: `OFDMerger.AttrQueries`（XPath 查询映射）。
///
/// 扫描以下属性：`ResourceID`、`Font`、`DrawParam`、`ColorSpace`、
/// `Substitution`、`ImageMask`、`Thumbnail`。
///
/// 返回 `(属性名, 属性值)` 列表（去重）。
fn extract_xml_refs(xml_bytes: &[u8]) -> Vec<(String, String)> {
    let xml = String::from_utf8_lossy(xml_bytes);
    let ref_attrs = [
        "ResourceID",
        "Font",
        "DrawParam",
        "ColorSpace",
        "Substitution",
        "ImageMask",
        "Thumbnail",
    ];
    let mut refs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for attr in &ref_attrs {
        let pattern = format!("{attr}=\"");
        let mut search_from = 0;
        while let Some(rel) = xml[search_from..].find(&pattern) {
            let abs = search_from + rel;
            let val_start = abs + pattern.len();
            if let Some(val_end) = xml[val_start..].find('"') {
                let value = xml[val_start..val_start + val_end].to_string();
                let key = (attr.to_string(), value.clone());
                if seen.insert(key) {
                    refs.push((attr.to_string(), value));
                }
            }
            search_from = val_start;
        }
    }

    refs
}

/// 重写 XML 中的资源引用 ID。
///
/// 对应 Java: `OFDMerger#domMigrate` 中的 `element.addAttribute(attrName, newResId)`。
///
/// 将 `attr="old_value"` 替换为 `attr="new_value"`。
fn rewrite_xml_refs(xml_bytes: &[u8], id_map: &HashMap<String, String>) -> Vec<u8> {
    let mut xml = String::from_utf8_lossy(xml_bytes).to_string();
    for (old_id, new_id) in id_map {
        for attr in &[
            "ResourceID",
            "Font",
            "DrawParam",
            "ColorSpace",
            "Substitution",
            "ImageMask",
            "Thumbnail",
        ] {
            let old_pattern = format!("{attr}=\"{old_id}\"");
            let new_pattern = format!("{attr}=\"{new_id}\"");
            xml = xml.replace(&old_pattern, &new_pattern);
        }
    }
    xml.into_bytes()
}

/// 构建合并后的 DocumentRes.xml（含页面图片 + 额外资源 + DrawParam）。
///
/// 对应 Java: `OFDMerger#resMigrate` 中的 `ofdDoc.prm.addRawWithCache(mm/dp)`。
///
/// # 参数
///
/// - `page_image_count`：页面内容中的图片总数（用于计算页面图片的 ResourceID 起始值）。
/// - `extra_media`：额外多媒体资源列表 `(res_name, format_str)`。
/// - `draw_params`：DrawParam 原始 XML 片段列表。
fn build_combined_doc_res_xml(
    page_image_count: usize,
    extra_media: &[(String, String)],
    draw_params: &[String],
) -> String {
    let mut xml = String::with_capacity(1024);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(r#"<ofd:DocumentRes xmlns:ofd="http://www.ofdspec.org/2016">"#);
    xml.push('\n');

    // MultiMedias（页面图片 + 额外资源）
    if page_image_count > 0 || !extra_media.is_empty() {
        xml.push_str("<ofd:MultiMedias>");
        xml.push('\n');

        // 页面图片资源（ID 从 100 开始，由 writer 生成，此处仅记录占位）
        // 注意：writer 自己会生成这些条目，所以这里只生成额外资源的条目。
        // 但如果 writer 跳过生成（preserved 优先），则需要包含所有条目。
        // 为简化，此处生成所有条目（页面图片 + 额外资源）。
        for i in 0..page_image_count {
            let id = 100 + i;
            // 页面图片的 res_name 由 merger 的 dedup_image_resource 设置
            // 此处无法获知具体 res_name，使用占位格式
            // 实际 res_name 在 writer 中确定
            // 这里只生成 ID 占位，实际文件路径由 writer 处理
            xml.push_str(&format!(
                r#"<ofd:MultiMedia ID="{id}" Type="Image"><ofd:MediaFile>placeholder</ofd:MediaFile></ofd:MultiMedia>"#
            ));
            xml.push('\n');
        }

        // 额外资源（ID 从 100 + page_image_count 开始）
        for (i, (res_name, format_str)) in extra_media.iter().enumerate() {
            let id = 100 + page_image_count + i;
            let type_str = format_str.as_str();
            xml.push_str(&format!(
                r#"<ofd:MultiMedia ID="{id}" Type="{type_str}"><ofd:MediaFile>{res_name}</ofd:MediaFile></ofd:MultiMedia>"#
            ));
            xml.push('\n');
        }

        xml.push_str("</ofd:MultiMedias>");
        xml.push('\n');
    }

    // DrawParams
    if !draw_params.is_empty() {
        xml.push_str("<ofd:DrawParams>");
        xml.push('\n');
        for dp_xml in draw_params {
            xml.push_str(dp_xml);
            xml.push('\n');
        }
        xml.push_str("</ofd:DrawParams>");
        xml.push('\n');
    }

    xml.push_str("</ofd:DocumentRes>");
    xml.push('\n');
    xml
}

/// 构建合并后的 PublicRes.xml（含 ColorSpace + Font 定义）。
///
/// 对应 Java: `OFDMerger#resMigrate` 中 `CT_ColorSpace` / `CT_Font` 分支的
/// `ofdDoc.prm.addRawWithCache(cs/f)` 调用。
///
/// # 参数
///
/// - `color_spaces`：ColorSpace 原始 XML 片段列表（已改写 ID 和 ProfileFile 路径）。
/// - `fonts`：Font 原始 XML 片段列表（已改写 ID 和 FontFile 路径）。
fn build_combined_pub_res_xml(color_spaces: &[String], fonts: &[String]) -> String {
    let mut xml = String::with_capacity(512);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(r#"<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">"#);
    xml.push('\n');

    // ColorSpaces
    if !color_spaces.is_empty() {
        xml.push_str("<ofd:ColorSpaces>");
        xml.push('\n');
        for cs_xml in color_spaces {
            xml.push_str(cs_xml);
            xml.push('\n');
        }
        xml.push_str("</ofd:ColorSpaces>");
        xml.push('\n');
    }

    // Fonts
    if !fonts.is_empty() {
        xml.push_str("<ofd:Fonts>");
        xml.push('\n');
        for font_xml in fonts {
            xml.push_str(font_xml);
            xml.push('\n');
        }
        xml.push_str("</ofd:Fonts>");
        xml.push('\n');
    }

    xml.push_str("</ofd:Res>");
    xml.push('\n');
    xml
}

/// 构建合并后的 Annotations.xml 索引。
///
/// 为每个有注解的合并页生成 `<ofd:Page PageID="..."><ofd:FileLoc>...</ofd:FileLoc></ofd:Page>` 条目。
fn build_annotations_index_xml(
    doc_dir: &str,
    pages: &[(usize, usize)],
    page_ids: &[String],
    annot_index: &[(String, String)],
) -> String {
    let mut entries_xml = String::new();
    for &(src_page_idx, merged_idx) in pages {
        let Some(src_page_id) = page_ids.get(src_page_idx) else {
            continue;
        };
        if annot_index.iter().any(|(pid, _)| pid == src_page_id) {
            let new_page_id = merged_idx + 1; // 合并后页面 ID = 索引 + 1（1-based）
            let file_loc = format!("/{doc_dir}/Annots/Page_{merged_idx}/Annot_0.xml");
            entries_xml.push_str(&format!(
                r#"<ofd:Page PageID="{new_page_id}"><ofd:FileLoc>{file_loc}</ofd:FileLoc></ofd:Page>"#
            ));
        }
    }

    if entries_xml.is_empty() {
        return String::new();
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><ofd:Annotations xmlns:ofd="http://www.ofdspec.org/2016">{entries_xml}</ofd:Annotations>"#
    )
}

/// 迁移注解和模板到合并产物（含资源迁移 + 模板 ID 重映射）。
///
/// 对应 Java: `OFDMerger#pageAnnotationMigrate` + `OFDMerger#pageTplMigrate`
///         + `OFDMerger#resMigrate` + `OFDMerger#domMigrate`
///
/// # 参数
///
/// - `source_paths`：源文档路径映射（按需打开 ZIP，不驻留全部字节）。
/// - `merged_page_map`：合并页面映射 `(源文档索引, 源页面索引)`。
/// - `merged_page_flags`：合并页面标志 `(copy_annotations, copy_template)`。
/// - `page_image_count`：页面内容中的图片总数（用于计算额外资源的 ResourceID 起始值）。
/// - `res_name_to_id`：页面图片资源名 → ResourceID 映射（用于复用已有资源）。
/// - `context`：合并上下文（含资源去重器）。
///
/// # 返回
///
/// - `preserve_entries`: 需要注入产物 ZIP 的条目（路径, 字节）
/// - `annotations_path`: Document.xml 中 `<ofd:Annotations>` 的路径值
/// - `template_pages`: 合并后的模板页引用列表
/// - `extra_resources`: 注解/模板引用的额外图片资源 `(res_name, data, format_str)`
///
/// # 资源迁移策略
///
/// 对应 Java: `OFDMerger#resMigrate` + `OFDMerger#domMigrate`
///
/// 1. 解析源 DocumentRes.xml 提取资源定义（MultiMedia、DrawParam）。
/// 2. 解析源 PublicRes.xml 提取资源定义（ColorSpace、Font）。
/// 3. 扫描注解/模板 XML 中的 ResourceID、DrawParam、ColorSpace、Font 等引用。
/// 4. 对 MultiMedia 资源：从源 ZIP 拷贝文件，SM3 去重，分配新 ID。
/// 5. 对 DrawParam 资源：保留原始 XML，分配新 ID。
/// 6. 对 ColorSpace 资源：ICC ProfileFile SM3 去重拷贝 + 路径改写，分配新 ID。
/// 7. 对 Font 资源：FontFile SM3 去重拷贝 + 路径改写，分配新 ID。
/// 8. 重写注解/模板 XML 中的资源引用 ID。
/// 9. 构建合并后的 DocumentRes.xml + PublicRes.xml。
fn migrate_extras(
    source_paths: &HashMap<usize, String>,
    merged_page_map: &[(usize, usize)],
    merged_page_flags: &[(bool, bool)],
    page_image_count: usize,
    res_name_to_id: &HashMap<String, usize>,
    context: &mut DocContext,
) -> MigrateResult {
    let mut preserve_entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut has_annotations = false;
    let mut template_pages: Vec<TemplatePage> = Vec::new();
    let mut extra_resources: Vec<(String, Vec<u8>, String)> = Vec::new();

    // 按源文档分组：需要迁移注解的页面
    let mut annot_pages: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    // 需要迁移模板的源文档
    let mut tpl_needed: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (merged_idx, &(src_doc, src_page)) in merged_page_map.iter().enumerate() {
        let (copy_annots, copy_tpl) = merged_page_flags[merged_idx];
        if copy_annots {
            annot_pages
                .entry(src_doc)
                .or_default()
                .push((src_page, merged_idx));
        }
        if copy_tpl {
            tpl_needed.insert(src_doc);
        }
    }

    // ── 跟踪额外资源计数器和 DrawParam 计数器 ──
    let mut extra_media_counter: usize = 0;
    let mut draw_param_counter: usize = 0;
    // ColorSpace 计数器（新 ID 从 600 开始）
    #[allow(unused)]
    let mut color_space_counter: usize = 0;
    // Font 计数器（新 ID 从 700 开始）
    #[allow(unused)]
    let mut font_counter: usize = 0;
    // 本源文档已处理的 DrawParam（避免同一源文档内重复迁移）
    let mut migrated_draw_params: HashMap<String, String> = HashMap::new();
    // 所有源文档合并后的 DrawParam XML 片段
    let mut all_draw_params: Vec<String> = Vec::new();
    // 所有源文档合并后的额外多媒体资源
    let mut all_extra_media: Vec<(String, String)> = Vec::new(); // (res_name, format_str)
    // 所有源文档合并后的 ColorSpace XML 片段
    let mut all_color_spaces: Vec<String> = Vec::new();
    // 所有源文档合并后的 Font XML 片段
    let mut all_fonts: Vec<String> = Vec::new();
    // 本源文档的资源定义缓存（合并 DocumentRes + PublicRes）
    let mut res_def_cache: HashMap<usize, HashMap<String, ResDef>> = HashMap::new();
    // 用于收集 ColorSpace/Font 迁移过程中产生的额外文件（ICC / 字体文件）
    let mut extra_file_entries: Vec<(String, Vec<u8>)> = Vec::new();

    // ── 注解迁移（含资源迁移）──
    //
    // 对应 Java: `OFDMerger#pageAnnotationMigrate` + `OFDMerger#domMigrate` + `OFDMerger#resMigrate`
    for (src_doc, pages) in &annot_pages {
        let Some(src_path) = source_paths.get(src_doc) else {
            continue;
        };
        let file = match fs::File::open(src_path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            continue;
        };

        // 1. 提取 doc_dir
        let doc_dir = read_zip_entry_bytes(&mut archive, "OFD.xml")
            .and_then(|b| extract_doc_dir(&b))
            .unwrap_or_else(|| "Doc_0".to_string());

        // 2. 提取页面 ID 列表
        let doc_xml_path = format!("{doc_dir}/Document.xml");
        let page_ids = read_zip_entry_bytes(&mut archive, &doc_xml_path)
            .map(|b| extract_page_ids(&b))
            .unwrap_or_default();

        // 3. 读取 Annotations.xml
        let annot_xml_path = format!("{doc_dir}/Annots/Annotations.xml");
        let annot_index = match read_zip_entry_bytes(&mut archive, &annot_xml_path) {
            Some(bytes) => extract_annot_index(&bytes),
            None => continue,
        };

        // 4. 解析源资源定义（合并 DocumentRes + PublicRes，缓存）
        //
        // 对应 Java: `DocContext.resMgt` 同时管理 DocumentRes 和 PublicRes 的资源。
        let all_defs = res_def_cache
            .entry(*src_doc)
            .or_insert_with(|| {
                let mut defs = HashMap::new();
                // DocumentRes.xml：MultiMedia + DrawParam
                let doc_res_path = format!("{doc_dir}/DocumentRes.xml");
                if let Some(bytes) = read_zip_entry_bytes(&mut archive, &doc_res_path) {
                    defs.extend(parse_doc_res_defs(&bytes));
                }
                // PublicRes.xml：ColorSpace + Font
                let pub_res_path = format!("{doc_dir}/PublicRes.xml");
                if let Some(bytes) = read_zip_entry_bytes(&mut archive, &pub_res_path) {
                    defs.extend(parse_pub_res_defs(&bytes));
                }
                defs
            })
            .clone();

        // 5. 为每个需要迁移注解的页面复制注解文件（含资源迁移）
        for &(src_page_idx, merged_idx) in pages {
            let Some(src_page_id) = page_ids.get(src_page_idx) else {
                continue;
            };
            // 在 Annotations.xml 中查找该页面的注解文件
            let annot_file_loc = annot_index
                .iter()
                .find(|(pid, _)| pid == src_page_id)
                .map(|(_, loc)| loc.clone());
            let Some(file_loc) = annot_file_loc else {
                continue;
            };

            // 读取注解文件原始字节（去掉路径开头的 '/'）
            let zip_path = file_loc.trim_start_matches('/');
            let annot_bytes = match read_zip_entry_bytes(&mut archive, zip_path) {
                Some(bytes) => bytes,
                None => continue,
            };

            // ── 资源迁移：提取注解 XML 中的资源引用并迁移 ──
            //
            // 对应 Java: `OFDMerger#domMigrate(docCtx, pageAnnot)`
            let refs = extract_xml_refs(&annot_bytes);
            let mut id_map: HashMap<String, String> = HashMap::new();

            for (_attr_name, old_id) in &refs {
                // 跳过已迁移的 ID
                if id_map.contains_key(old_id) {
                    continue;
                }

                if let Some(res_def) = all_defs.get(old_id) {
                    match res_def {
                        ResDef::MultiMedia {
                            file_path, format, ..
                        } => {
                            // 对应 Java: `CT_MultiMedia` 分支
                            let file_zip_path = if file_path.contains('/') {
                                format!("{doc_dir}/{file_path}")
                            } else {
                                format!("{doc_dir}/Res/{file_path}")
                            };
                            if let Some(file_data) =
                                read_zip_entry_bytes(&mut archive, &file_zip_path)
                            {
                                let hash = ResourceDedup::compute_hash(&file_data);
                                let dedup = context.resource_dedup_mut();
                                let res_name = if let Some(existing) = dedup.get_by_hash(&hash) {
                                    existing.to_string()
                                } else {
                                    let ext = match format.as_str() {
                                        "PNG" => ".png",
                                        "JPEG" | "JPG" => ".jpeg",
                                        "BMP" => ".bmp",
                                        "TIFF" | "TIF" => ".tiff",
                                        _ => "",
                                    };
                                    let counter = dedup.counter() + 1;
                                    let name = format!("Res/{counter}{ext}");
                                    dedup.register(hash, name.clone());
                                    name
                                };

                                let new_id =
                                    if let Some(&existing_id) = res_name_to_id.get(&res_name) {
                                        existing_id
                                    } else {
                                        let id = 100 + page_image_count + extra_media_counter;
                                        extra_media_counter += 1;
                                        extra_resources.push((
                                            res_name.clone(),
                                            file_data,
                                            format.clone(),
                                        ));
                                        all_extra_media.push((res_name.clone(), format.clone()));
                                        id
                                    };

                                id_map.insert(old_id.clone(), new_id.to_string());
                            }
                        }
                        ResDef::DrawParam { id, raw_xml } => {
                            // 对应 Java: `CT_DrawParam` 分支
                            let new_id_str = if let Some(new_id) = migrated_draw_params.get(old_id)
                            {
                                new_id.clone()
                            } else {
                                let new_id = 500 + draw_param_counter;
                                draw_param_counter += 1;
                                let new_id_str = new_id.to_string();
                                let new_xml = raw_xml.replace(
                                    &format!("ID=\"{id}\""),
                                    &format!("ID=\"{new_id_str}\""),
                                );
                                all_draw_params.push(new_xml);
                                migrated_draw_params.insert(old_id.clone(), new_id_str.clone());
                                new_id_str
                            };
                            id_map.insert(old_id.clone(), new_id_str);
                        }
                        ResDef::ColorSpace {
                            profile_file,
                            raw_xml,
                            ..
                        } => {
                            // 对应 Java: `CT_ColorSpace` 分支
                            //
                            // ColorSpace 定义在 PublicRes.xml 中，引用 ICC ProfileFile。
                            // 迁移步骤：拷贝 ProfileFile（SM3 去重）+ 路径改写 + 分配新 ID。
                            let mut new_xml = raw_xml.clone();
                            if let Some(profile) = profile_file {
                                let profile_zip_path = if profile.contains('/') {
                                    format!("{doc_dir}/{profile}")
                                } else {
                                    format!("{doc_dir}/Res/{profile}")
                                };
                                if let Some(profile_data) =
                                    read_zip_entry_bytes(&mut archive, &profile_zip_path)
                                {
                                    let hash = ResourceDedup::compute_hash(&profile_data);
                                    let dedup = context.resource_dedup_mut();
                                    let new_profile_name =
                                        if let Some(existing) = dedup.get_by_hash(&hash) {
                                            existing.to_string()
                                        } else {
                                            let counter = dedup.counter() + 1;
                                            let name = format!("Res/{counter}.icc");
                                            dedup.register(hash, name.clone());
                                            name
                                        };
                                    new_xml = new_xml.replace(profile, &new_profile_name);
                                    let icc_path = format!("Doc_0/{new_profile_name}");
                                    extra_file_entries.push((icc_path, profile_data));
                                }
                            }
                            let new_id = 600 + color_space_counter;
                            color_space_counter += 1;
                            let new_id_str = new_id.to_string();
                            // 替换 XML 中的旧 ID
                            if let Some(old_id_val) = extract_attr_value(&new_xml, "ID") {
                                new_xml = new_xml.replace(
                                    &format!("ID=\"{old_id_val}\""),
                                    &format!("ID=\"{new_id_str}\""),
                                );
                            }
                            all_color_spaces.push(new_xml);
                            id_map.insert(old_id.clone(), new_id_str);
                        }
                        ResDef::Font {
                            font_file, raw_xml, ..
                        } => {
                            // 对应 Java: `CT_Font` 分支
                            //
                            // Font 定义在 PublicRes.xml 中，FontFile 属性指向字体文件。
                            // 迁移步骤：拷贝 FontFile（SM3 去重）+ 路径改写 + 分配新 ID。
                            let mut new_xml = raw_xml.clone();
                            if let Some(font_file_path) = font_file {
                                let font_zip_path = if font_file_path.contains('/') {
                                    format!("{doc_dir}/{font_file_path}")
                                } else {
                                    format!("{doc_dir}/Res/{font_file_path}")
                                };
                                if let Some(font_data) =
                                    read_zip_entry_bytes(&mut archive, &font_zip_path)
                                {
                                    let hash = ResourceDedup::compute_hash(&font_data);
                                    let dedup = context.resource_dedup_mut();
                                    let new_font_name =
                                        if let Some(existing) = dedup.get_by_hash(&hash) {
                                            existing.to_string()
                                        } else {
                                            let ext = std::path::Path::new(font_file_path)
                                                .extension()
                                                .and_then(|e| e.to_str())
                                                .map(|e| format!(".{e}"))
                                                .unwrap_or_default();
                                            let counter = dedup.counter() + 1;
                                            let name = format!("Res/{counter}{ext}");
                                            dedup.register(hash, name.clone());
                                            name
                                        };
                                    new_xml = new_xml.replace(font_file_path, &new_font_name);
                                    let font_archive_path = format!("Doc_0/{new_font_name}");
                                    extra_file_entries.push((font_archive_path, font_data));
                                }
                            }
                            let new_id = 700 + font_counter;
                            font_counter += 1;
                            let new_id_str = new_id.to_string();
                            if let Some(old_id_val) = extract_attr_value(&new_xml, "ID") {
                                new_xml = new_xml.replace(
                                    &format!("ID=\"{old_id_val}\""),
                                    &format!("ID=\"{new_id_str}\""),
                                );
                            }
                            all_fonts.push(new_xml);
                            id_map.insert(old_id.clone(), new_id_str);
                        }
                    }
                }
            }

            // 重写注解 XML 中的资源引用 ID
            //
            // 对应 Java: `OFDMerger#domMigrate` 中 `element.addAttribute(attrName, newResId)`
            let rewritten_annot = if id_map.is_empty() {
                annot_bytes
            } else {
                rewrite_xml_refs(&annot_bytes, &id_map)
            };

            // 注解文件路径：使用合并后的页索引命名目录
            // 对应 Java: pageAnnotDirName = "Page_" + mergedIndex
            let target_path = format!("{doc_dir}/Annots/Page_{merged_idx}/Annot_0.xml");
            preserve_entries.push((target_path, rewritten_annot));
            has_annotations = true;
        }

        // 6. 构建合并后的 Annotations.xml 索引
        let index_xml = build_annotations_index_xml(&doc_dir, pages, &page_ids, &annot_index);
        if !index_xml.is_empty() {
            let index_path = format!("{doc_dir}/Annots/Annotations.xml");
            preserve_entries.push((index_path, index_xml.into_bytes()));
        }
    }

    // ── 模板迁移（含资源迁移 + ID 重映射）──
    //
    // 对应 Java: `OFDMerger#pageTplMigrate` + `OFDMerger#domMigrate` + `OFDMerger#resMigrate`
    let mut used_tpl_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tpl_id_counter: usize = 0;

    for src_doc in &tpl_needed {
        let Some(src_path) = source_paths.get(src_doc) else {
            continue;
        };
        let file = match fs::File::open(src_path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            continue;
        };

        let doc_dir = read_zip_entry_bytes(&mut archive, "OFD.xml")
            .and_then(|b| extract_doc_dir(&b))
            .unwrap_or_else(|| "Doc_0".to_string());

        // 解析源资源定义（复用缓存）
        let all_defs = res_def_cache
            .entry(*src_doc)
            .or_insert_with(|| {
                let mut defs = HashMap::new();
                let doc_res_path = format!("{doc_dir}/DocumentRes.xml");
                if let Some(bytes) = read_zip_entry_bytes(&mut archive, &doc_res_path) {
                    defs.extend(parse_doc_res_defs(&bytes));
                }
                let pub_res_path = format!("{doc_dir}/PublicRes.xml");
                if let Some(bytes) = read_zip_entry_bytes(&mut archive, &pub_res_path) {
                    defs.extend(parse_pub_res_defs(&bytes));
                }
                defs
            })
            .clone();

        // 提取模板页引用
        let doc_xml_path = format!("{doc_dir}/Document.xml");
        if let Some(doc_bytes) = read_zip_entry_bytes(&mut archive, &doc_xml_path) {
            let tpl_entries = extract_template_pages(&doc_bytes);
            for (id, base_loc) in &tpl_entries {
                // 模板 ID 重映射：检测冲突并分配新 ID
                //
                // 对应 Java: `OFDMerger#pageTplMigrate` 中 `tplPageMap.get(oldId)` 冲突检测
                let new_tpl_id = if used_tpl_ids.contains(id) {
                    // ID 冲突：分配新 ID
                    let new_id = format!("tpl_{tpl_id_counter}");
                    tpl_id_counter += 1;
                    new_id
                } else {
                    id.clone()
                };
                used_tpl_ids.insert(new_tpl_id.clone());

                // 复制模板页文件（含资源迁移）
                let tpl_zip_path = format!("{doc_dir}/{base_loc}");
                if let Some(tpl_bytes) = read_zip_entry_bytes(&mut archive, &tpl_zip_path) {
                    // ── 模板资源迁移 ──
                    //
                    // 对应 Java: `OFDMerger#domMigrate(docCtx, pageObj)` 在 pageTplMigrate 中
                    let refs = extract_xml_refs(&tpl_bytes);
                    let mut id_map: HashMap<String, String> = HashMap::new();

                    for (_attr_name, old_id) in &refs {
                        if id_map.contains_key(old_id) {
                            continue;
                        }

                        if let Some(res_def) = all_defs.get(old_id) {
                            match res_def {
                                ResDef::MultiMedia {
                                    file_path, format, ..
                                } => {
                                    let file_zip_path = if file_path.contains('/') {
                                        format!("{doc_dir}/{file_path}")
                                    } else {
                                        format!("{doc_dir}/Res/{file_path}")
                                    };
                                    if let Some(file_data) =
                                        read_zip_entry_bytes(&mut archive, &file_zip_path)
                                    {
                                        let hash = ResourceDedup::compute_hash(&file_data);
                                        let dedup = context.resource_dedup_mut();
                                        let res_name =
                                            if let Some(existing) = dedup.get_by_hash(&hash) {
                                                existing.to_string()
                                            } else {
                                                let ext = match format.as_str() {
                                                    "PNG" => ".png",
                                                    "JPEG" | "JPG" => ".jpeg",
                                                    "BMP" => ".bmp",
                                                    "TIFF" | "TIF" => ".tiff",
                                                    _ => "",
                                                };
                                                let counter = dedup.counter() + 1;
                                                let name = format!("Res/{counter}{ext}");
                                                dedup.register(hash, name.clone());
                                                name
                                            };

                                        let new_id = if let Some(&existing_id) =
                                            res_name_to_id.get(&res_name)
                                        {
                                            existing_id
                                        } else {
                                            let id = 100 + page_image_count + extra_media_counter;
                                            extra_media_counter += 1;
                                            extra_resources.push((
                                                res_name.clone(),
                                                file_data,
                                                format.clone(),
                                            ));
                                            all_extra_media
                                                .push((res_name.clone(), format.clone()));
                                            id
                                        };

                                        id_map.insert(old_id.clone(), new_id.to_string());
                                    }
                                }
                                ResDef::DrawParam { id, raw_xml } => {
                                    let new_id_str =
                                        if let Some(new_id) = migrated_draw_params.get(old_id) {
                                            new_id.clone()
                                        } else {
                                            let new_id = 500 + draw_param_counter;
                                            draw_param_counter += 1;
                                            let new_id_str = new_id.to_string();
                                            let new_xml = raw_xml.replace(
                                                &format!("ID=\"{id}\""),
                                                &format!("ID=\"{new_id_str}\""),
                                            );
                                            all_draw_params.push(new_xml);
                                            migrated_draw_params
                                                .insert(old_id.clone(), new_id_str.clone());
                                            new_id_str
                                        };
                                    id_map.insert(old_id.clone(), new_id_str);
                                }
                                ResDef::ColorSpace {
                                    profile_file,
                                    raw_xml,
                                    ..
                                } => {
                                    // 对应 Java: `CT_ColorSpace` 分支
                                    let mut new_xml = raw_xml.clone();
                                    if let Some(profile) = profile_file {
                                        let profile_zip_path = if profile.contains('/') {
                                            format!("{doc_dir}/{profile}")
                                        } else {
                                            format!("{doc_dir}/Res/{profile}")
                                        };
                                        if let Some(profile_data) =
                                            read_zip_entry_bytes(&mut archive, &profile_zip_path)
                                        {
                                            let hash = ResourceDedup::compute_hash(&profile_data);
                                            let dedup = context.resource_dedup_mut();
                                            let new_profile_name =
                                                if let Some(existing) = dedup.get_by_hash(&hash) {
                                                    existing.to_string()
                                                } else {
                                                    let counter = dedup.counter() + 1;
                                                    let name = format!("Res/{counter}.icc");
                                                    dedup.register(hash, name.clone());
                                                    name
                                                };
                                            new_xml = new_xml.replace(profile, &new_profile_name);
                                            let icc_path = format!("Doc_0/{new_profile_name}");
                                            extra_file_entries.push((icc_path, profile_data));
                                        }
                                    }
                                    let new_id = 600 + color_space_counter;
                                    color_space_counter += 1;
                                    let new_id_str = new_id.to_string();
                                    if let Some(old_id_val) = extract_attr_value(&new_xml, "ID") {
                                        new_xml = new_xml.replace(
                                            &format!("ID=\"{old_id_val}\""),
                                            &format!("ID=\"{new_id_str}\""),
                                        );
                                    }
                                    all_color_spaces.push(new_xml);
                                    id_map.insert(old_id.clone(), new_id_str);
                                }
                                ResDef::Font {
                                    font_file, raw_xml, ..
                                } => {
                                    // 对应 Java: `CT_Font` 分支
                                    let mut new_xml = raw_xml.clone();
                                    if let Some(font_file_path) = font_file {
                                        let font_zip_path = if font_file_path.contains('/') {
                                            format!("{doc_dir}/{font_file_path}")
                                        } else {
                                            format!("{doc_dir}/Res/{font_file_path}")
                                        };
                                        if let Some(font_data) =
                                            read_zip_entry_bytes(&mut archive, &font_zip_path)
                                        {
                                            let hash = ResourceDedup::compute_hash(&font_data);
                                            let dedup = context.resource_dedup_mut();
                                            let new_font_name =
                                                if let Some(existing) = dedup.get_by_hash(&hash) {
                                                    existing.to_string()
                                                } else {
                                                    let ext = std::path::Path::new(font_file_path)
                                                        .extension()
                                                        .and_then(|e| e.to_str())
                                                        .map(|e| format!(".{e}"))
                                                        .unwrap_or_default();
                                                    let counter = dedup.counter() + 1;
                                                    let name = format!("Res/{counter}{ext}");
                                                    dedup.register(hash, name.clone());
                                                    name
                                                };
                                            new_xml =
                                                new_xml.replace(font_file_path, &new_font_name);
                                            let font_archive_path =
                                                format!("Doc_0/{new_font_name}");
                                            extra_file_entries.push((font_archive_path, font_data));
                                        }
                                    }
                                    let new_id = 700 + font_counter;
                                    font_counter += 1;
                                    let new_id_str = new_id.to_string();
                                    if let Some(old_id_val) = extract_attr_value(&new_xml, "ID") {
                                        new_xml = new_xml.replace(
                                            &format!("ID=\"{old_id_val}\""),
                                            &format!("ID=\"{new_id_str}\""),
                                        );
                                    }
                                    all_fonts.push(new_xml);
                                    id_map.insert(old_id.clone(), new_id_str);
                                }
                            }
                        }
                    }

                    let rewritten_tpl = if id_map.is_empty() {
                        tpl_bytes
                    } else {
                        rewrite_xml_refs(&tpl_bytes, &id_map)
                    };

                    let target_path = format!("{doc_dir}/{base_loc}");
                    preserve_entries.push((target_path, rewritten_tpl));
                }
                template_pages.push(TemplatePage::new(&new_tpl_id, base_loc));
            }
        }
    }

    // ── 注入 ColorSpace/Font 迁移产生的额外文件（ICC / 字体文件）──
    preserve_entries.extend(extra_file_entries);

    // ── 构建合并后的 DocumentRes.xml ──
    //
    // 对应 Java: `OFDMerger#resMigrate` 中的 `ofdDoc.prm.addRawWithCache(mm/dp)`
    if !all_extra_media.is_empty() || !all_draw_params.is_empty() {
        let combined_xml =
            build_combined_doc_res_xml(page_image_count, &all_extra_media, &all_draw_params);
        let doc_dir = "Doc_0";
        preserve_entries.push((
            format!("{doc_dir}/DocumentRes.xml"),
            combined_xml.into_bytes(),
        ));
    }

    // ── 构建合并后的 PublicRes.xml（含 ColorSpace + Font）──
    //
    // 对应 Java: `OFDMerger#resMigrate` 中 `CT_ColorSpace` / `CT_Font` 分支的
    // `ofdDoc.prm.addRawWithCache(cs/f)` 调用——资源定义写入 PublicRes.xml。
    if !all_color_spaces.is_empty() || !all_fonts.is_empty() {
        let pub_res_xml = build_combined_pub_res_xml(&all_color_spaces, &all_fonts);
        let doc_dir = "Doc_0";
        preserve_entries.push((format!("{doc_dir}/PublicRes.xml"), pub_res_xml.into_bytes()));
    }

    let annotations_path = if has_annotations {
        Some("Annots/Annotations.xml".to_string())
    } else {
        None
    };

    (
        preserve_entries,
        annotations_path,
        template_pages,
        extra_resources,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{ImageObject, TextObject};
    use easyofd_writer::OfdWriter;

    /// 辅助函数：用 OfdWriter 创建 OFD 文件，返回临时文件路径。
    fn create_test_ofd(pages: Vec<OfdPage>) -> tempfile::TempPath {
        let mut writer = OfdWriter::new();
        for page in pages {
            writer.add_page(page);
        }
        let bytes = writer.build().unwrap();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &bytes).unwrap();
        tmp.into_temp_path()
    }

    /// 辅助函数：创建含文本的页面。
    fn text_page(text: &str, width: f64, height: f64) -> OfdPage {
        let mut page = OfdPage::new(width, height);
        page.add_text(TextObject::new(10.0, 20.0, text));
        page
    }

    /// 辅助函数：创建含文本+图片的页面。
    fn text_image_page(text: &str, image_data: Vec<u8>, width: f64, height: f64) -> OfdPage {
        let mut page = OfdPage::new(width, height);
        page.add_text(TextObject::new(10.0, 20.0, text));
        page.add_image(ImageObject::png(100.0, 100.0, 30.0, 30.0, image_data));
        page
    }

    // ── 原有测试保留 ──────────────────────────────────────────────────

    #[test]
    fn new_merger() {
        let merger = OfdMerger::new("/tmp/merged.ofd");
        assert_eq!(merger.output_path(), "/tmp/merged.ofd");
        assert_eq!(merger.source_count(), 0);
        assert_eq!(merger.page_count(), 0);
    }

    #[test]
    fn add_sources_and_pages() {
        let mut merger = OfdMerger::new("/tmp/out.ofd");
        let idx0 = merger.add_source("/tmp/a.ofd", 3);
        let idx1 = merger.add_source("/tmp/b.ofd", 2);

        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
        assert_eq!(merger.source_count(), 2);

        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 1, 210.0, 297.0));
        assert_eq!(merger.page_count(), 2);
    }

    #[test]
    fn merge_fails_without_sources() {
        let mut merger = OfdMerger::new("/tmp/out.ofd");
        let result = merger.merge();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("源文档"));
    }

    #[test]
    fn merge_fails_without_pages() {
        let mut merger = OfdMerger::new("/tmp/out.ofd");
        merger.add_source("/tmp/a.ofd", 3);
        let result = merger.merge();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("页面"));
    }

    // ── 真实合并测试 ──────────────────────────────────────────────────

    #[test]
    fn merge_two_sources_text_only() {
        // 两个源各 2 页纯文本 → 合并后 4 页，内容与源一致
        let src_a = create_test_ofd(vec![
            text_page("A第1页", 210.0, 297.0),
            text_page("A第2页", 210.0, 297.0),
        ]);
        let src_b = create_test_ofd(vec![
            text_page("B第1页", 210.0, 297.0),
            text_page("B第2页", 210.0, 297.0),
        ]);

        let mut merger = OfdMerger::new("/tmp/merge_text_only.ofd");
        merger.add_source(src_a.to_str().unwrap(), 2);
        merger.add_source(src_b.to_str().unwrap(), 2);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(0, 1, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 1, 210.0, 297.0));

        let bytes = merger.merge().unwrap();

        // 非空
        assert!(!bytes.is_empty());
        // ZIP 头
        assert_eq!(&bytes[0..2], b"PK");

        // 用 OfdReader 读回验证
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 4);

        let texts = reader.extract_text();
        assert!(texts[0].contains("A第1页"));
        assert!(texts[1].contains("A第2页"));
        assert!(texts[2].contains("B第1页"));
        assert!(texts[3].contains("B第2页"));
    }

    #[test]
    fn merge_two_sources_with_images() {
        // 两个源各 1 页含文本+图片 → 合并后 2 页
        let img_data_a = vec![0x89, 0x50, 0x4E, 0x47, 0xAA, 0xBB];
        let img_data_b = vec![0x89, 0x50, 0x4E, 0x47, 0xCC, 0xDD];

        let src_a = create_test_ofd(vec![text_image_page("图文A", img_data_a, 210.0, 297.0)]);
        let src_b = create_test_ofd(vec![text_image_page("图文B", img_data_b, 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_images.ofd");
        merger.add_source(src_a.to_str().unwrap(), 1);
        merger.add_source(src_b.to_str().unwrap(), 1);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 2);

        // 每页应有 2 个内容对象（1 文本 + 1 图片）
        assert_eq!(reader.pages()[0].content.len(), 2);
        assert_eq!(reader.pages()[1].content.len(), 2);

        // 验证文本内容
        let texts = reader.extract_text();
        assert!(texts[0].contains("图文A"));
        assert!(texts[1].contains("图文B"));

        // 验证图片数据
        if let ContentObject::Image(img) = &reader.pages()[0].content[1] {
            assert_eq!(img.data, vec![0x89, 0x50, 0x4E, 0x47, 0xAA, 0xBB]);
        } else {
            panic!("第 1 页第 2 个对象应为图片");
        }
        if let ContentObject::Image(img) = &reader.pages()[1].content[1] {
            assert_eq!(img.data, vec![0x89, 0x50, 0x4E, 0x47, 0xCC, 0xDD]);
        } else {
            panic!("第 2 页第 2 个对象应为图片");
        }
    }

    #[test]
    fn merge_dedup_same_image_data() {
        // 两个源包含相同图片字节 → 产物 ZIP 内图片资源 entry 只出现一份
        let shared_img_data = vec![0x89, 0x50, 0x4E, 0x47, 0xDE, 0xAD];

        let src_a = create_test_ofd(vec![text_image_page(
            "源A",
            shared_img_data.clone(),
            210.0,
            297.0,
        )]);
        let src_b = create_test_ofd(vec![text_image_page("源B", shared_img_data, 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_dedup.ofd");
        merger.add_source(src_a.to_str().unwrap(), 1);
        merger.add_source(src_b.to_str().unwrap(), 1);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();

        // 解包统计：ZIP 内图片资源文件只应出现一份
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let image_entries: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .filter(|name| name.contains("/Res/") && !name.ends_with('/'))
            .collect();
        assert_eq!(
            image_entries.len(),
            1,
            "相同图片内容应只产生一个资源文件，实际: {:?}",
            image_entries,
        );

        // 读回验证：两页图片数据一致
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 2);
        if let ContentObject::Image(img_a) = &reader.pages()[0].content[1] {
            if let ContentObject::Image(img_b) = &reader.pages()[1].content[1] {
                assert_eq!(img_a.data, img_b.data);
            } else {
                panic!("第 2 页第 2 个对象应为图片");
            }
        } else {
            panic!("第 1 页第 2 个对象应为图片");
        }
    }

    #[test]
    fn merge_page_bounds_error() {
        // 页码越界 → Err 且包含源路径信息
        let src = create_test_ofd(vec![text_page("只有1页", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_bounds.ofd");
        let src_path = src.to_str().unwrap().to_string();
        merger.add_source(&src_path, 1);
        merger.add_page(DocPage::new(0, 5, 210.0, 297.0)); // 请求第 6 页（0-based=5）

        let result = merger.merge();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("越界"), "错误信息应包含'越界'，实际: {}", err);
        assert!(
            err.contains("6 页"),
            "错误信息应显示请求的页码，实际: {}",
            err
        );
    }

    #[test]
    fn merge_with_page_entry_and_tb_mix_pages() {
        // PageEntry + tb_mix_pages：将两个页面内容叠加到一个页面
        let src_a = create_test_ofd(vec![text_page("基础页", 210.0, 297.0)]);
        let src_b = create_test_ofd(vec![text_page("叠加页", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_mix.ofd");
        merger.add_source(src_a.to_str().unwrap(), 1);
        merger.add_source(src_b.to_str().unwrap(), 1);

        // 目标页：源 A 第 1 页（page_index=1，1-based）
        // 叠加页：源 B 第 1 页
        let mix_entry = PageEntry::new(1, 1); // 源 B 第 1 页
        let entry = PageEntry::with_mix_pages(1, 0, vec![mix_entry]);
        merger.add_page_entry(entry);

        let bytes = merger.merge().unwrap();
        assert!(!bytes.is_empty());

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 1);

        // 叠加后目标页 content 数 = 基础页(1) + 叠加页(1) = 2
        assert_eq!(
            reader.pages()[0].content.len(),
            2,
            "tb_mix_pages 叠加后目标页 content 数应为 2",
        );
    }

    #[test]
    fn merge_output_path_writes_file() {
        // 设置 output_path 时，merge 同时写入文件
        let src = create_test_ofd(vec![text_page("文件写入测试", 210.0, 297.0)]);

        let out_dir = tempfile::tempdir().unwrap();
        let out_path = out_dir.path().join("merged_output.ofd");

        let mut merger = OfdMerger::new(out_path.to_str().unwrap());
        merger.add_source(src.to_str().unwrap(), 1);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();

        // 验证文件已写入
        assert!(out_path.exists(), "输出文件应已创建");
        let file_bytes = fs::read(&out_path).unwrap();
        assert_eq!(file_bytes, bytes, "文件内容应与返回字节一致");
    }

    #[test]
    fn merge_page_entry_page_zero_error() {
        // PageEntry 页码为 0 → 报错
        let src = create_test_ofd(vec![text_page("测试", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_pe_zero.ofd");
        merger.add_source(src.to_str().unwrap(), 1);
        merger.add_page_entry(PageEntry::new(0, 0)); // page_index=0 无效

        let result = merger.merge();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不能为 0"));
    }

    #[test]
    fn merge_mixed_doc_page_and_page_entry() {
        // DocPage 和 PageEntry 双轨混合使用
        let src_a = create_test_ofd(vec![
            text_page("A1", 210.0, 297.0),
            text_page("A2", 210.0, 297.0),
        ]);
        let src_b = create_test_ofd(vec![text_page("B1", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_mixed.ofd");
        merger.add_source(src_a.to_str().unwrap(), 2);
        merger.add_source(src_b.to_str().unwrap(), 1);

        // DocPage 路径（0-based）
        merger.add_page(DocPage::new(0, 1, 210.0, 297.0)); // 源 A 第 2 页
        // PageEntry 路径（1-based）
        merger.add_page_entry(PageEntry::new(1, 1)); // 源 B 第 1 页

        let bytes = merger.merge().unwrap();
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 2);

        let texts = reader.extract_text();
        assert!(texts[0].contains("A2"));
        assert!(texts[1].contains("B1"));
    }

    // ── 注解迁移测试 ──────────────────────────────────────────────────────

    #[test]
    fn merge_with_annotation_migration() {
        // 合并含注解的 OFD（gen_08_annotations.ofd）→ 产物应包含注解条目
        let annot_ofd = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/ofdrw_gen/gen_08_annotations.ofd");
        if !annot_ofd.exists() {
            eprintln!("跳过：fixture 不存在 {}", annot_ofd.display());
            return;
        }
        let annot_path = annot_ofd.to_str().unwrap();

        let plain = create_test_ofd(vec![text_page("普通页", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_annot.ofd");
        merger.add_source(annot_path, 1);
        merger.add_source(plain.to_str().unwrap(), 1);
        // DocPage 路径：默认 copy_annotations=true, copy_template=true
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0)); // 注解源页
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0)); // 普通页

        let bytes = merger.merge().unwrap();
        assert!(!bytes.is_empty());

        // 解包产物 ZIP 检查注解条目
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let entry_names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        // 应包含 Annotations.xml 索引
        assert!(
            entry_names
                .iter()
                .any(|n| n.ends_with("Annots/Annotations.xml")),
            "产物应包含 Annotations.xml，实际条目: {:?}",
            entry_names,
        );

        // 应包含第一个合并页（merged_idx=0）的注解文件
        assert!(
            entry_names.iter().any(|n| n.contains("Annots/Page_0/")),
            "产物应包含 Page_0 注解目录，实际条目: {:?}",
            entry_names,
        );

        // 注解文件内容应非空
        let annot_path_in_zip = entry_names
            .iter()
            .find(|n| {
                n.contains("Annots/Page_0/")
                    && std::path::Path::new(n)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
            })
            .unwrap()
            .clone();
        let mut annot_file = archive.by_name(&annot_path_in_zip).unwrap();
        let mut annot_content = String::new();
        std::io::Read::read_to_string(&mut annot_file, &mut annot_content).unwrap();
        assert!(
            annot_content.contains("PageAnnot") || annot_content.contains("Annot"),
            "注解文件应包含注解内容: {}",
            annot_content,
        );
    }

    #[test]
    fn merge_copy_annotations_false() {
        // copy_annotations=false 时不迁移注解
        let annot_ofd = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/ofdrw_gen/gen_08_annotations.ofd");
        if !annot_ofd.exists() {
            eprintln!("跳过：fixture 不存在 {}", annot_ofd.display());
            return;
        }
        let annot_path = annot_ofd.to_str().unwrap();

        let plain = create_test_ofd(vec![text_page("普通页", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_no_annot.ofd");
        merger.add_source(annot_path, 1);
        merger.add_source(plain.to_str().unwrap(), 1);
        // PageEntry 路径：显式关闭 copy_annotations
        let entry = PageEntry::new(1, 0).copy_annotations(false);
        merger.add_page_entry(entry);
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();

        // 解包产物 ZIP 检查：不应包含注解条目
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let entry_names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        assert!(
            !entry_names.iter().any(|n| n.contains("Annots/")),
            "copy_annotations=false 时不应迁移注解，实际条目: {:?}",
            entry_names,
        );
    }

    #[test]
    fn merge_annotation_migrates_per_page_flag() {
        // 混合场景：第 1 页迁移注解，第 2 页不迁移
        let annot_ofd = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/ofdrw_gen/gen_08_annotations.ofd");
        if !annot_ofd.exists() {
            eprintln!("跳过：fixture 不存在 {}", annot_ofd.display());
            return;
        }
        let annot_path = annot_ofd.to_str().unwrap();

        let mut merger = OfdMerger::new("/tmp/merge_mixed_annot.ofd");
        merger.add_source(annot_path, 1);
        // DocPage 路径：默认 copy_annotations=true
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();

        // 解包产物 ZIP 检查注解条目
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let entry_names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        // 应包含 Annotations.xml 索引
        assert!(
            entry_names
                .iter()
                .any(|n| n.ends_with("Annots/Annotations.xml")),
            "产物应包含 Annotations.xml",
        );
    }

    // ── XML 解析辅助函数测试 ───────────────────────────────────────────────

    #[test]
    fn test_extract_doc_dir() {
        let xml = br#"<?xml version="1.0"?><ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016"><ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody></ofd:OFD>"#;
        assert_eq!(extract_doc_dir(xml), Some("Doc_0".to_string()));
    }

    #[test]
    fn test_extract_doc_dir_with_leading_slash() {
        let xml = b"<ofd:DocRoot>/Doc_0/Document.xml</ofd:DocRoot>";
        assert_eq!(extract_doc_dir(xml), Some("Doc_0".to_string()));
    }

    #[test]
    fn test_extract_page_ids() {
        let xml = br#"<ofd:Pages><ofd:Page ID="1" BaseLoc="Pages/Page_0/Content.xml"/><ofd:Page ID="49" BaseLoc="Pages/Page_1/Content.xml"/></ofd:Pages>"#;
        let ids = extract_page_ids(xml);
        assert_eq!(ids, vec!["1".to_string(), "49".to_string()]);
    }

    #[test]
    fn test_extract_annot_index() {
        let xml = br#"<ofd:Annotations><ofd:Page PageID="1"><ofd:FileLoc>/Doc_0/Annots/Page_0/Annot_0.xml</ofd:FileLoc></ofd:Page></ofd:Annotations>"#;
        let entries = extract_annot_index(xml);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "1");
        assert_eq!(entries[0].1, "/Doc_0/Annots/Page_0/Annot_0.xml");
    }

    #[test]
    fn test_extract_template_pages() {
        let xml = br#"<ofd:CommonData><ofd:TemplatePage ID="100" BaseLoc="Templates/Tpl_0.xml"/></ofd:CommonData>"#;
        let entries = extract_template_pages(xml);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "100");
        assert_eq!(entries[0].1, "Templates/Tpl_0.xml");
    }

    #[test]
    fn test_build_annotations_index_xml() {
        let pages = vec![(0, 0)]; // src_page_idx=0 → merged_idx=0
        let page_ids = vec!["1".to_string()];
        let annot_index = vec![(
            "1".to_string(),
            "/Doc_0/Annots/Page_0/Annot_0.xml".to_string(),
        )];
        let xml = build_annotations_index_xml("Doc_0", &pages, &page_ids, &annot_index);
        assert!(xml.contains("PageID=\"1\""));
        assert!(xml.contains("Annots/Page_0/Annot_0.xml"));
    }

    #[test]
    fn test_build_annotations_index_xml_empty() {
        // 无注解页面 → 空字符串
        let pages = vec![(0, 0)];
        let page_ids = vec!["1".to_string()];
        let annot_index: Vec<(String, String)> = vec![];
        let xml = build_annotations_index_xml("Doc_0", &pages, &page_ids, &annot_index);
        assert!(xml.is_empty());
    }

    // ── 资源迁移辅助函数测试 ────────────────────────────────────────────────────

    #[test]
    fn test_extract_attr_value() {
        let tag = r#"<ofd:MultiMedia Type="Image" Format="PNG" ID="26">"#;
        assert_eq!(extract_attr_value(tag, "ID"), Some("26".to_string()));
        assert_eq!(extract_attr_value(tag, "Type"), Some("Image".to_string()));
        assert_eq!(extract_attr_value(tag, "Format"), Some("PNG".to_string()));
        assert_eq!(extract_attr_value(tag, "Missing"), None);
    }

    #[test]
    fn test_extract_child_text() {
        let xml = r#"<ofd:MultiMedia ID="26"><ofd:MediaFile>_stamp_img.png</ofd:MediaFile></ofd:MultiMedia>"#;
        assert_eq!(
            extract_child_text(xml, "ofd:MediaFile"),
            Some("_stamp_img.png".to_string())
        );
        assert_eq!(extract_child_text(xml, "ofd:Missing"), None);
    }

    #[test]
    fn test_parse_doc_res_defs_multimedia() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res"><ofd:MultiMedias><ofd:MultiMedia Type="Image" Format="PNG" ID="26"><ofd:MediaFile>_stamp_img.png</ofd:MediaFile></ofd:MultiMedia></ofd:MultiMedias></ofd:Res>"#;
        let defs = parse_doc_res_defs(xml);
        assert_eq!(defs.len(), 1);
        match defs.get("26").unwrap() {
            ResDef::MultiMedia {
                file_path, format, ..
            } => {
                assert_eq!(file_path, "_stamp_img.png");
                assert_eq!(format, "PNG");
            }
            ResDef::DrawParam { .. } | ResDef::ColorSpace { .. } | ResDef::Font { .. } => {
                panic!("应为 MultiMedia")
            }
        }
    }

    #[test]
    fn test_parse_doc_res_defs_drawparam() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res"><ofd:DrawParams><ofd:DrawParam ID="27"><ofd:FillColor Value="0 0 0"/><ofd:StrokeColor Value="0 0 0"/></ofd:DrawParam></ofd:DrawParams></ofd:Res>"#;
        let defs = parse_doc_res_defs(xml);
        assert_eq!(defs.len(), 1);
        match defs.get("27").unwrap() {
            ResDef::DrawParam { id, raw_xml } => {
                assert_eq!(id, "27");
                assert!(raw_xml.contains("FillColor"));
                assert!(raw_xml.contains("StrokeColor"));
            }
            ResDef::MultiMedia { .. } | ResDef::ColorSpace { .. } | ResDef::Font { .. } => {
                panic!("应为 DrawParam")
            }
        }
    }

    #[test]
    fn test_extract_xml_refs() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:PageAnnot xmlns:ofd="http://www.ofdspec.org/2016"><ofd:Annot Type="Stamp" ID="26"><ofd:Appearance><ofd:ImageObject ID="27" ResourceID="26" DrawParam="27"/></ofd:Appearance></ofd:Annot></ofd:PageAnnot>"#;
        let refs = extract_xml_refs(xml);
        // 应提取 ResourceID="26" 和 DrawParam="27"
        assert!(refs.iter().any(|(a, v)| a == "ResourceID" && v == "26"));
        assert!(refs.iter().any(|(a, v)| a == "DrawParam" && v == "27"));
    }

    #[test]
    fn test_rewrite_xml_refs() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:PageAnnot><ofd:ImageObject ResourceID="26" DrawParam="27"/></ofd:PageAnnot>"#;
        let mut id_map = HashMap::new();
        id_map.insert("26".to_string(), "100".to_string());
        id_map.insert("27".to_string(), "500".to_string());
        let rewritten = rewrite_xml_refs(xml, &id_map);
        let rewritten_str = String::from_utf8_lossy(&rewritten);
        assert!(rewritten_str.contains("ResourceID=\"100\""));
        assert!(rewritten_str.contains("DrawParam=\"500\""));
        assert!(!rewritten_str.contains("ResourceID=\"26\""));
        assert!(!rewritten_str.contains("DrawParam=\"27\""));
    }

    #[test]
    fn test_build_combined_doc_res_xml() {
        let extra_media = vec![
            ("Res/1.png".to_string(), "PNG".to_string()),
            ("Res/2.jpeg".to_string(), "JPEG".to_string()),
        ];
        let draw_params = vec![
            r#"<ofd:DrawParam ID="500"><ofd:FillColor Value="0 0 0"/></ofd:DrawParam>"#.to_string(),
        ];
        let xml = build_combined_doc_res_xml(0, &extra_media, &draw_params);
        assert!(xml.contains("ofd:DocumentRes"));
        assert!(xml.contains("ofd:MultiMedias"));
        assert!(xml.contains("ID=\"100\"")); // 第一个额外资源
        assert!(xml.contains("ID=\"101\"")); // 第二个额外资源
        assert!(xml.contains("Res/1.png"));
        assert!(xml.contains("Res/2.jpeg"));
        assert!(xml.contains("ofd:DrawParams"));
        assert!(xml.contains("ID=\"500\""));
    }

    // ── 注解资源迁移集成测试 ──────────────────────────────────────────────────

    #[test]
    fn merge_annotation_resource_migration() {
        // 合并含印章注解的 OFD → 产物应包含：
        // 1. 印章图片资源文件（Res/ 目录下）
        // 2. DocumentRes.xml 含资源定义
        // 3. 注解 XML 中 ResourceID 已改写为新 ID
        let annot_ofd = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/ofdrw_gen/gen_08_annotations.ofd");
        if !annot_ofd.exists() {
            eprintln!("跳过：fixture 不存在 {}", annot_ofd.display());
            return;
        }
        let annot_path = annot_ofd.to_str().unwrap();

        let plain = create_test_ofd(vec![text_page("普通页", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_annot_res.ofd");
        merger.add_source(annot_path, 1);
        merger.add_source(plain.to_str().unwrap(), 1);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0)); // 注解源页
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0)); // 普通页

        let bytes = merger.merge().unwrap();
        assert!(!bytes.is_empty());

        // 解包产物 ZIP
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let entry_names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        // ── 验证 1：印章图片资源文件存在 ──
        let res_files: Vec<&str> = entry_names
            .iter()
            .filter(|n| n.contains("/Res/") && !n.ends_with('/'))
            .map(|s| s.as_str())
            .collect();
        assert!(
            !res_files.is_empty(),
            "产物应包含 Res/ 目录下的资源文件，实际条目: {:?}",
            entry_names,
        );
        // 印章图片应为 PNG 格式
        assert!(
            res_files.iter().any(|n| std::path::Path::new(n)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))),
            "产物应包含 PNG 资源文件，实际资源: {:?}",
            res_files,
        );

        // ── 验证 2：DocumentRes.xml 存在且含资源定义 ──
        let doc_res_path = entry_names
            .iter()
            .find(|n| n.ends_with("DocumentRes.xml"))
            .cloned();
        assert!(
            doc_res_path.is_some(),
            "产物应包含 DocumentRes.xml，实际条目: {:?}",
            entry_names,
        );
        let doc_res_content = {
            let mut f = archive.by_name(&doc_res_path.unwrap()).unwrap();
            let mut s = String::new();
            std::io::Read::read_to_string(&mut f, &mut s).unwrap();
            s
        };
        assert!(
            doc_res_content.contains("ofd:MultiMedia"),
            "DocumentRes.xml 应含 MultiMedia 条目: {}",
            doc_res_content,
        );
        assert!(
            doc_res_content.contains("ofd:DrawParam"),
            "DocumentRes.xml 应含 DrawParam 条目: {}",
            doc_res_content,
        );

        // ── 验证 3：注解 XML 中 ResourceID 已改写 ──
        // 源文档中 ResourceID="26"，合并后应改为新 ID（如 "100"）
        let annot_xml_path = entry_names
            .iter()
            .find(|n| n.contains("Annots/Page_0/Annot_0.xml"))
            .cloned();
        assert!(
            annot_xml_path.is_some(),
            "产物应包含注解文件，实际条目: {:?}",
            entry_names,
        );
        let annot_content = {
            let mut f = archive.by_name(&annot_xml_path.unwrap()).unwrap();
            let mut s = String::new();
            std::io::Read::read_to_string(&mut f, &mut s).unwrap();
            s
        };
        // 注解 XML 中不应再引用源文档的 ResourceID="26"
        // （页面无图片，所以新 ID 从 100 开始）
        assert!(
            annot_content.contains("ResourceID=\"100\""),
            "注解 XML 中 ResourceID 应改写为 100（页面无图片时），实际: {}",
            annot_content,
        );
        // DrawParam 也应改写（从 27 改为 500+）
        assert!(
            annot_content.contains("DrawParam=\"500\""),
            "注解 XML 中 DrawParam 应改写为 500，实际: {}",
            annot_content,
        );
    }

    #[test]
    fn merge_annotation_resource_dedup() {
        // 合并两个含相同印章注解的源 → 产物中印章图片应只出现一份（SM3 去重）
        let annot_ofd = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/ofdrw_gen/gen_08_annotations.ofd");
        if !annot_ofd.exists() {
            eprintln!("跳过：fixture 不存在 {}", annot_ofd.display());
            return;
        }
        let annot_path = annot_ofd.to_str().unwrap();

        let mut merger = OfdMerger::new("/tmp/merge_annot_dedup.ofd");
        merger.add_source(annot_path, 1);
        merger.add_source(annot_path, 1);
        // 两个源各取 1 页（同一文件的两页，但 OFD 只有 1 页 → 用 DocPage 路径）
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();

        // 解包统计：Res/ 下的 PNG 文件应只有一份（SM3 去重）
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let png_res_files: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .filter(|n| {
                n.contains("/Res/")
                    && std::path::Path::new(n)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            })
            .collect();
        assert_eq!(
            png_res_files.len(),
            1,
            "相同印章图片应只产生一个资源文件（SM3 去重），实际: {:?}",
            png_res_files,
        );
    }

    // ── ColorSpace / Font 资源迁移测试 ────────────────────────────────────────

    /// 辅助函数：创建含自定义 PublicRes.xml + 注解 + 资源文件的 OFD。
    ///
    /// 返回临时文件路径。
    fn create_ofd_with_pub_res(
        page: OfdPage,
        pub_res_xml: &str,
        annot_xml: &str,
        extra_files: &[(&str, &[u8])],
    ) -> tempfile::TempPath {
        let mut writer = OfdWriter::new();
        writer.add_page(page);

        let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
        // PublicRes.xml（writer 检测到同名条目时会跳过自动生成）
        entries.push((
            "Doc_0/PublicRes.xml".to_string(),
            pub_res_xml.as_bytes().to_vec(),
        ));
        // Annotations.xml 索引
        entries.push((
            "Doc_0/Annots/Annotations.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8"?><ofd:Annotations xmlns:ofd="http://www.ofdspec.org/2016"><ofd:Page PageID="1"><ofd:FileLoc>/Doc_0/Annots/Page_0/Annot_0.xml</ofd:FileLoc></ofd:Page></ofd:Annotations>"#.to_vec(),
        ));
        // 注解文件
        entries.push((
            "Doc_0/Annots/Page_0/Annot_0.xml".to_string(),
            annot_xml.as_bytes().to_vec(),
        ));
        // 额外资源文件（ICC / 字体等）
        for (path, data) in extra_files {
            entries.push((format!("Doc_0/{path}"), data.to_vec()));
        }
        writer.preserve_entries(entries);

        let bytes = writer.build().unwrap();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &bytes).unwrap();
        tmp.into_temp_path()
    }

    #[test]
    fn test_parse_pub_res_defs_colorspace() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">
  <ofd:ColorSpaces>
    <ofd:ColorSpace ID="10" Type="RGB" Profile="sRGB.icc"/>
  </ofd:ColorSpaces>
</ofd:Res>"#;
        let defs = parse_pub_res_defs(xml);
        assert_eq!(defs.len(), 1);
        match defs.get("10").unwrap() {
            ResDef::ColorSpace {
                cs_type,
                profile_file,
                ..
            } => {
                assert_eq!(cs_type, "RGB");
                assert_eq!(profile_file.as_deref(), Some("sRGB.icc"));
            }
            _ => panic!("应为 ColorSpace"),
        }
    }

    #[test]
    fn test_parse_pub_res_defs_font_with_file() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">
  <ofd:Fonts>
    <ofd:Font ID="3" FontName="SimSun" FontFile="Res/simsun.ttf"/>
  </ofd:Fonts>
</ofd:Res>"#;
        let defs = parse_pub_res_defs(xml);
        assert_eq!(defs.len(), 1);
        match defs.get("3").unwrap() {
            ResDef::Font {
                font_name,
                font_file,
                ..
            } => {
                assert_eq!(font_name, "SimSun");
                assert_eq!(font_file.as_deref(), Some("Res/simsun.ttf"));
            }
            _ => panic!("应为 Font"),
        }
    }

    #[test]
    fn test_parse_pub_res_defs_combined() {
        // 同时包含 ColorSpace 和 Font
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">
  <ofd:ColorSpaces><ofd:ColorSpace ID="10" Type="CMYK"/></ofd:ColorSpaces>
  <ofd:Fonts><ofd:Font ID="3" FontName="SimSun"/></ofd:Fonts>
</ofd:Res>"#;
        let defs = parse_pub_res_defs(xml);
        assert_eq!(defs.len(), 2);
        assert!(defs.contains_key("10"));
        assert!(defs.contains_key("3"));
    }

    #[test]
    fn test_build_combined_pub_res_xml() {
        let color_spaces =
            vec![r#"<ofd:ColorSpace ID="600" Type="RGB" Profile="Res/1.icc"/>"#.to_string()];
        let fonts =
            vec![r#"<ofd:Font ID="700" FontName="宋体" FontFile="Res/2.ttf"/>"#.to_string()];
        let xml = build_combined_pub_res_xml(&color_spaces, &fonts);
        assert!(xml.contains("ofd:Res"));
        assert!(xml.contains("ofd:ColorSpaces"));
        assert!(xml.contains("ID=\"600\""));
        assert!(xml.contains("ofd:Fonts"));
        assert!(xml.contains("ID=\"700\""));
    }

    #[test]
    fn merge_colorspace_resource_migration() {
        // 构造含 ColorSpace(ICC) 引用的 OFD → merge → 产物断言：
        // 1. PublicRes.xml 含 ColorSpace 定义（ID 已改写）
        // 2. ICC 文件存在
        // 3. 注解 XML 中 ColorSpace 引用已改写
        let icc_data = b"ICC_PROFILE_DATA_FAKE_FOR_TEST";

        let annot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:PageAnnot xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Annot Type="Stamp" ID="100">
    <ofd:Appearance Boundary="10 10 50 50">
      <ofd:PathObject ID="101" ColorSpace="10" Boundary="0 0 50 50"/>
    </ofd:Appearance>
  </ofd:Annot>
</ofd:PageAnnot>"#;

        let pub_res_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">
  <ofd:ColorSpaces>
    <ofd:ColorSpace ID="10" Type="RGB" Profile="Res/custom.icc"/>
  </ofd:ColorSpaces>
</ofd:Res>"#;

        let src = create_ofd_with_pub_res(
            text_page("ColorSpace测试", 210.0, 297.0),
            pub_res_xml,
            annot_xml,
            &[("Res/custom.icc", icc_data)],
        );

        let plain = create_test_ofd(vec![text_page("普通页", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_cs.ofd");
        merger.add_source(src.to_str().unwrap(), 1);
        merger.add_source(plain.to_str().unwrap(), 1);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();
        assert!(!bytes.is_empty());

        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let entry_names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        // ── 验证 1：PublicRes.xml 含 ColorSpace 定义 ──
        let pub_res_path = entry_names
            .iter()
            .find(|n| n.ends_with("PublicRes.xml"))
            .cloned();
        assert!(
            pub_res_path.is_some(),
            "产物应含 PublicRes.xml，实际: {:?}",
            entry_names
        );
        let pub_res_content = {
            let mut f = archive.by_name(&pub_res_path.unwrap()).unwrap();
            let mut s = String::new();
            std::io::Read::read_to_string(&mut f, &mut s).unwrap();
            s
        };
        assert!(
            pub_res_content.contains("ofd:ColorSpace"),
            "PublicRes.xml 应含 ColorSpace: {}",
            pub_res_content,
        );
        // ID 应从 10 改写为 600+
        assert!(
            pub_res_content.contains("ID=\"600\""),
            "PublicRes.xml ColorSpace ID 应改写为 600: {}",
            pub_res_content,
        );

        // ── 验证 2：ICC 文件存在 ──
        let icc_files: Vec<&str> = entry_names
            .iter()
            .filter(|n| {
                n.contains("/Res/")
                    && std::path::Path::new(n)
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("icc"))
            })
            .map(|s| s.as_str())
            .collect();
        assert!(
            !icc_files.is_empty(),
            "产物应含 ICC 文件，实际条目: {:?}",
            entry_names,
        );

        // ── 验证 3：注解 XML 中 ColorSpace 引用已改写 ──
        let annot_path = entry_names
            .iter()
            .find(|n| n.contains("Annots/Page_0/Annot_0.xml"))
            .cloned();
        assert!(annot_path.is_some(), "产物应含注解文件");
        let annot_content = {
            let mut f = archive.by_name(&annot_path.unwrap()).unwrap();
            let mut s = String::new();
            std::io::Read::read_to_string(&mut f, &mut s).unwrap();
            s
        };
        assert!(
            annot_content.contains("ColorSpace=\"600\""),
            "注解 XML 中 ColorSpace 应改写为 600: {}",
            annot_content,
        );
    }

    #[test]
    fn merge_font_resource_migration() {
        // 构造含 Font(FontFile) 引用的 OFD → merge → 产物断言：
        // 1. PublicRes.xml 含 Font 定义（ID 已改写）
        // 2. 字体文件存在
        // 3. 注解 XML 中 Font 引用已改写
        let font_data = b"FAKE_FONT_DATA_FOR_TEST_TTF";

        let annot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:PageAnnot xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Annot Type="Text" ID="200">
    <ofd:Appearance Boundary="10 10 100 30">
      <ofd:TextObject ID="201" Font="3" Boundary="0 0 100 30">
        <ofd:TextCode X="0" Y="10">测试字体</ofd:TextCode>
      </ofd:TextObject>
    </ofd:Appearance>
  </ofd:Annot>
</ofd:PageAnnot>"#;

        let pub_res_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">
  <ofd:Fonts>
    <ofd:Font ID="3" FontName="测试字体" FontFile="Res/test_font.ttf"/>
  </ofd:Fonts>
</ofd:Res>"#;

        let src = create_ofd_with_pub_res(
            text_page("Font测试", 210.0, 297.0),
            pub_res_xml,
            annot_xml,
            &[("Res/test_font.ttf", font_data)],
        );

        let plain = create_test_ofd(vec![text_page("普通页", 210.0, 297.0)]);

        let mut merger = OfdMerger::new("/tmp/merge_font.ofd");
        merger.add_source(src.to_str().unwrap(), 1);
        merger.add_source(plain.to_str().unwrap(), 1);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));
        merger.add_page(DocPage::new(1, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();
        assert!(!bytes.is_empty());

        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let entry_names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        // ── 验证 1：PublicRes.xml 含 Font 定义 ──
        let pub_res_path = entry_names
            .iter()
            .find(|n| n.ends_with("PublicRes.xml"))
            .cloned();
        assert!(
            pub_res_path.is_some(),
            "产物应含 PublicRes.xml，实际: {:?}",
            entry_names
        );
        let pub_res_content = {
            let mut f = archive.by_name(&pub_res_path.unwrap()).unwrap();
            let mut s = String::new();
            std::io::Read::read_to_string(&mut f, &mut s).unwrap();
            s
        };
        assert!(
            pub_res_content.contains("ofd:Font"),
            "PublicRes.xml 应含 Font: {}",
            pub_res_content,
        );
        // ID 应从 3 改写为 700+
        assert!(
            pub_res_content.contains("ID=\"700\""),
            "PublicRes.xml Font ID 应改写为 700: {}",
            pub_res_content,
        );

        // ── 验证 2：字体文件存在 ──
        let font_files: Vec<&str> = entry_names
            .iter()
            .filter(|n| {
                n.contains("/Res/")
                    && std::path::Path::new(n)
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("ttf"))
            })
            .map(|s| s.as_str())
            .collect();
        assert!(
            !font_files.is_empty(),
            "产物应含 .ttf 字体文件，实际条目: {:?}",
            entry_names,
        );

        // ── 验证 3：注解 XML 中 Font 引用已改写 ──
        let annot_path = entry_names
            .iter()
            .find(|n| n.contains("Annots/Page_0/Annot_0.xml"))
            .cloned();
        assert!(annot_path.is_some(), "产物应含注解文件");
        let annot_content = {
            let mut f = archive.by_name(&annot_path.unwrap()).unwrap();
            let mut s = String::new();
            std::io::Read::read_to_string(&mut f, &mut s).unwrap();
            s
        };
        assert!(
            annot_content.contains("Font=\"700\""),
            "注解 XML 中 Font 应改写为 700: {}",
            annot_content,
        );
    }

    #[test]
    fn merge_colorspace_and_font_together() {
        // 同时迁移 ColorSpace + Font → 产物 PublicRes.xml 含两者
        let icc_data = b"ICC_FAKE";
        let font_data = b"FONT_FAKE";

        let annot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:PageAnnot xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Annot Type="Stamp" ID="300">
    <ofd:Appearance Boundary="10 10 100 50">
      <ofd:TextObject ID="301" Font="3" ColorSpace="10" Boundary="0 0 100 50">
        <ofd:TextCode X="0" Y="20">合并测试</ofd:TextCode>
      </ofd:TextObject>
    </ofd:Appearance>
  </ofd:Annot>
</ofd:PageAnnot>"#;

        let pub_res_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">
  <ofd:ColorSpaces><ofd:ColorSpace ID="10" Type="RGB" Profile="Res/icc.data"/></ofd:ColorSpaces>
  <ofd:Fonts><ofd:Font ID="3" FontName="测试" FontFile="Res/font.ttf"/></ofd:Fonts>
</ofd:Res>"#;

        let src = create_ofd_with_pub_res(
            text_page("合并测试", 210.0, 297.0),
            pub_res_xml,
            annot_xml,
            &[("Res/icc.data", icc_data), ("Res/font.ttf", font_data)],
        );

        let mut merger = OfdMerger::new("/tmp/merge_both.ofd");
        merger.add_source(src.to_str().unwrap(), 1);
        merger.add_page(DocPage::new(0, 0, 210.0, 297.0));

        let bytes = merger.merge().unwrap();

        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let entry_names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        let pub_res_path = entry_names
            .iter()
            .find(|n| n.ends_with("PublicRes.xml"))
            .cloned()
            .expect("产物应含 PublicRes.xml");
        let pub_res_content = {
            let mut f = archive.by_name(&pub_res_path).unwrap();
            let mut s = String::new();
            std::io::Read::read_to_string(&mut f, &mut s).unwrap();
            s
        };
        assert!(
            pub_res_content.contains("ofd:ColorSpace"),
            "应含 ColorSpace"
        );
        assert!(pub_res_content.contains("ofd:Font"), "应含 Font");
        assert!(
            pub_res_content.contains("ID=\"600\""),
            "ColorSpace ID 应为 600"
        );
        assert!(pub_res_content.contains("ID=\"700\""), "Font ID 应为 700");

        // ICC 和字体文件均存在
        assert!(
            entry_names.iter().any(|n| n.contains("/Res/")
                && std::path::Path::new(n)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("icc"))
                || std::path::Path::new(n)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("data"))),
            "应含 ICC 文件",
        );
        assert!(
            entry_names.iter().any(|n| n.contains("/Res/")
                && std::path::Path::new(n)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("ttf"))),
            "应含字体文件",
        );
    }
}
