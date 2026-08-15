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
//! ### 未覆盖项
//!
//! | Java 功能             | Rust 状态     | 说明                                     |
//! |-----------------------|---------------|------------------------------------------|
//! | 模板页迁移            | 未覆盖        | 模型层无 TemplatePage 对应物             |
//! | 注释迁移              | 未覆盖        | 模型层无 Annotations 对应物              |
//! | DrawParam 迁移        | 未覆盖        | 模型层无 DrawParam 对应物                |
//! | Font 文件迁移         | 部分覆盖      | 字体名保留，字体文件需 writer 嵌入       |
//! | DOM 对象 ID 重分配    | 不需要        | writer 重新生成所有 ID                   |
//! | `copyTemplate` 开关   | 尊重但无操作  | 模型层无模板，开关无效                   |
//! | `copyAnnotations` 开关| 尊重但无操作  | 模型层无注释，开关无效                   |

use std::collections::HashMap;
use std::fs;

use super::resource_dedup::ResourceDedup;
use super::{BareOFDDoc, DocContext, DocPage, PageEntry};
use easyofd_core::{ContentObject, ImageObject, OfdPage};
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;

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

        // ── 预提取 PageEntry 描述（含 tb_mix_pages）──
        #[allow(clippy::type_complexity)]
        let entry_descs: Vec<(usize, usize, Vec<(usize, usize)>)> = self
            .page_entries
            .iter()
            .map(|e| {
                let mix = e
                    .tb_mix_pages
                    .iter()
                    .map(|m| (m.doc_ctx_index, m.page_index))
                    .collect();
                (e.doc_ctx_index, e.page_index, mix)
            })
            .collect();

        // ── 缓存已加载的源文档字节，避免重复读取 ──
        let mut source_cache: HashMap<usize, Vec<u8>> = HashMap::new();

        // ── 收集合并后的页面 ──
        let mut merged_pages: Vec<OfdPage> = Vec::new();

        // ── 处理 DocPage 列表（page_index 从 0 开始）──
        for (source_index, page_index) in page_descs {
            let page_data = self.load_source_page(source_index, page_index, &mut source_cache)?;
            merged_pages.push(page_data);
        }

        // ── 处理 PageEntry 列表（page_index 从 1 开始）──
        for (doc_ctx_index, page_index, mix_pages) in entry_descs {
            // PageEntry.page_index 从 1 开始，转换为 0-based 索引
            let zero_based = page_index.checked_sub(1).ok_or_else(|| {
                let src = self.context.source_path(doc_ctx_index).unwrap_or("未知");
                format!("源文档 '{src}' 的页码不能为 0（PageEntry 页码从 1 开始）")
            })?;

            let mut page_data =
                self.load_source_page(doc_ctx_index, zero_based, &mut source_cache)?;

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
                    self.load_source_page(mix_ctx_index, mix_zero_based, &mut source_cache)?;
                // 将混合页的内容对象追加到目标页尾部
                page_data.content.extend(mix_page.content);
            }

            merged_pages.push(page_data);
        }

        // ── 生成新文档元数据 ──
        //
        // 对应 Java: `BareOFDDoc` 构造时生成新 DocID（UUID.randomUUID()）
        let metadata = easyofd_core::OfdMetadata {
            doc_id: Some(generate_doc_id()),
            ..easyofd_core::OfdMetadata::default()
        };

        // ── 构建输出 OFD ──
        let mut writer = OfdWriter::new();
        writer.set_metadata(metadata);
        writer.add_pages(merged_pages);

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
    /// 读取源文件、解析 OFD、提取页面，并对图片资源执行 SM3 去重。
    ///
    /// # 参数
    ///
    /// - `source_index`：源文档索引。
    /// - `page_index`：页面索引（从 0 开始）。
    /// - `reader_cache`：源文件字节缓存。
    fn load_source_page(
        &mut self,
        source_index: usize,
        page_index: usize,
        reader_cache: &mut HashMap<usize, Vec<u8>>,
    ) -> Result<OfdPage, String> {
        // 克隆路径字符串，释放对 self.context 的不可变借用，
        // 以便后续 dedup_image_resource 可以可变借用 self.context。
        let src_path = self.resolve_source_path(source_index)?.to_string();

        // 读取并缓存源文件字节
        if let std::collections::hash_map::Entry::Vacant(entry) = reader_cache.entry(source_index) {
            let bytes =
                fs::read(&src_path).map_err(|e| format!("读取源文档 '{}' 失败: {e}", src_path))?;
            entry.insert(bytes);
        }
        let bytes = &reader_cache[&source_index];

        // 解析 OFD
        let reader = OfdReader::from_bytes(bytes)
            .map_err(|e| format!("解析源文档 '{}' 失败: {e}", src_path))?;

        // 页码越界检查
        let total_pages = reader.pages().len();
        if page_index >= total_pages {
            return Err(format!(
                "源文档 '{}' 页码越界：请求第 {} 页，但文档共 {} 页",
                src_path,
                page_index + 1, // 显示为 1-based 页码
                total_pages,
            ));
        }

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
}
