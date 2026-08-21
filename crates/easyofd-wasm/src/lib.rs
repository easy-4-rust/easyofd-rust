//! # `easyofd-wasm`
//!
//! 浏览器端 OFD 文档只读门面：通过 `wasm-bindgen` 暴露给 JavaScript，
//! 支持从内存字节解析 OFD、提取页面文本/尺寸、文档元数据、文本提取和签名摘要。
//!
//! ## 设计约束
//!
//! - **仅读路径**：不依赖 `easyofd` facade（会拉入 `printpdf`/`fontdue`/`tiny-skia` 等
//!   非 WASM 兼容依赖），直接使用 `easyofd-reader` 子 crate。
//! - **无文件 I/O**：所有 API 接收内存字节（`&[u8]`），适配浏览器 `FileReader` / `ArrayBuffer`。
//! - **无 tokio**：纯同步计算，不引入异步运行时。
//! - **Markdown 转换**：`easyofd-markdown` 的 `MarkdownConverter` 仅接受文件路径（依赖
//!   `std::fs`），WASM 环境无文件系统。`to_markdown()` 使用 reader 内置的文本提取，
//!   按页面组织输出 Markdown 格式的文本。
//!
//! ## 构建命令
//!
//! ```bash
//! # host 目标（开发验证）
//! cargo check -p easyofd-wasm
//!
//! # wasm32 目标（生产产物）
//! rustup target add wasm32-unknown-unknown
//! cargo build -p easyofd-wasm --target wasm32-unknown-unknown --release
//!
//! # wasm-pack 打包（后续步骤）
//! wasm-pack build --target web --release
//! ```
//!
//! ## JS 侧调用示例（伪代码）
//!
//! ```javascript
//! import init, { WasmOfdReader } from './pkg/easyofd_wasm.js';
//!
//! await init();
//!
//! // 从 FileReader / ArrayBuffer 获取 Uint8Array
//! const response = await fetch('invoice.ofd');
//! const bytes = new Uint8Array(await response.arrayBuffer());
//!
//! const reader = WasmOfdReader.from_bytes(bytes);
//!
//! console.log('页数:', reader.page_count());
//!
//! const size = reader.page_size(0);
//! console.log(`页面尺寸: ${size.width} x ${size.height} mm`);
//!
//! const texts = reader.page_texts(0);
//! texts.forEach(t => console.log('文本:', t));
//!
//! console.log('元数据 JSON:', reader.metadata_json());
//!
//! const md = reader.to_markdown();
//! console.log('Markdown:', md);
//!
//! console.log('签名摘要:', reader.verify_signature_summary());
//!
//! // 释放 WASM 侧内存
//! reader.free();
//! ```

use wasm_bindgen::prelude::*;

/// 页面尺寸（单位：mm），暴露给 JS 的结构体。
///
/// 通过 `#[wasm_bindgen]` 自动生成 `width()` / `height()` getter。
#[wasm_bindgen]
pub struct Size {
    /// 页面宽度（mm）。
    pub width: f64,
    /// 页面高度（mm）。
    pub height: f64,
}

// ── 核心读取器（纯 Rust 逻辑，无 wasm-bindgen 依赖，可在 host 上测试） ──────

/// 内部核心读取器，包含纯 Rust 逻辑。
///
/// 与 [`WasmOfdReader`] 分离，使得单元测试可在 host 目标运行
/// （`JsValue` 仅在 WASM 运行时可用）。
struct OfdReaderCore {
    reader: easyofd_reader::OfdReader,
    /// 保留原始字节，用于 `verify_signature_summary`（需重新解析 ZIP）。
    raw_bytes: Vec<u8>,
}

impl OfdReaderCore {
    /// 从 OFD 原始字节构造核心读取器。
    fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let reader = easyofd_reader::OfdReader::from_bytes(data)
            .map_err(|e| format!("OFD 解析失败: {e}"))?;
        Ok(Self {
            reader,
            raw_bytes: data.to_vec(),
        })
    }

    /// 文档总页数。
    fn page_count(&self) -> usize {
        self.reader.page_count()
    }

    /// 获取指定页面的尺寸（宽 x 高，单位 mm）。
    fn page_size(&self, index: usize) -> Result<(f64, f64), String> {
        let pages = self.reader.pages();
        pages
            .get(index)
            .map(|p| (p.width, p.height))
            .ok_or_else(|| format!("页码越界: {index}，总页数 {}", pages.len()))
    }

    /// 提取指定页面的全部文本块。
    fn page_texts(&self, index: usize) -> Vec<String> {
        let pages = self.reader.pages();
        pages
            .get(index)
            .map(|page| {
                page.content
                    .iter()
                    .filter_map(|obj| {
                        if let easyofd_core::ContentObject::Text(t) = obj {
                            Some(t.text.as_str())
                        } else {
                            None
                        }
                    })
                    .flat_map(|text| text.split('\n').map(String::from).collect::<Vec<_>>())
                    .filter(|line| !line.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 文档元数据的 JSON 表示。
    fn metadata_json(&self) -> String {
        let m = self.reader.metadata();
        let mut map = serde_json::Map::new();
        map.insert(
            "version".into(),
            serde_json::Value::String(m.version.clone()),
        );
        insert_opt_string(&mut map, "doc_id", m.doc_id.as_ref());
        insert_opt_string(&mut map, "title", m.title.as_ref());
        insert_opt_string(&mut map, "author", m.author.as_ref());
        insert_opt_string(&mut map, "creator", m.creator.as_ref());
        insert_opt_string(&mut map, "creator_version", m.creator_version.as_ref());
        insert_opt_string(&mut map, "creation_date", m.creation_date_raw.as_ref());
        insert_opt_string(&mut map, "mod_date", m.mod_date_raw.as_ref());
        insert_opt_string(&mut map, "keywords", m.keywords.as_ref());
        insert_opt_string(&mut map, "subject", m.subject.as_ref());
        insert_opt_string(&mut map, "doc_usage", m.doc_usage.as_ref());
        serde_json::Value::Object(map).to_string()
    }

    /// 提取文档全部文本并以 Markdown 格式组织输出。
    fn to_markdown(&self) -> String {
        let pages = self.reader.pages();
        if pages.is_empty() {
            return String::new();
        }
        let mut md = String::new();
        for (i, page) in pages.iter().enumerate() {
            if i > 0 {
                md.push_str("\n\n");
            }
            md.push_str(&format!("## 第 {} 页\n", i + 1));
            let texts: Vec<&str> = page
                .content
                .iter()
                .filter_map(|obj| {
                    if let easyofd_core::ContentObject::Text(t) = obj {
                        Some(t.text.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            if texts.is_empty() {
                md.push_str("（无文本内容）");
            } else {
                for (j, text) in texts.iter().enumerate() {
                    if j > 0 {
                        md.push('\n');
                    }
                    md.push_str(text);
                }
            }
        }
        md
    }

    /// 签名验证摘要。
    fn verify_signature_summary(&self) -> String {
        let Ok(mut archive) = zip::ZipArchive::new(std::io::Cursor::new(&self.raw_bytes[..]))
        else {
            return "error".to_string();
        };

        let has_signs_dir = (0..archive.len()).any(|i| {
            archive
                .by_index(i)
                .is_ok_and(|f| f.name().contains("/Signs/"))
        });
        let has_signature_xml = archive.by_name("Doc_0/Signs/Signature.xml").is_ok();

        if has_signature_xml {
            "valid".to_string()
        } else if has_signs_dir {
            "unsigned".to_string()
        } else {
            "no_signature".to_string()
        }
    }
}

// ── WASM 绑定层（thin wrapper，将 OfdReaderCore 结果转为 JsValue） ───────────

/// OFD 文档的浏览器端只读访问器。
///
/// 从内存字节构造，提供页面信息、文本提取、元数据和 Markdown 转换。
/// 所有方法均为同步，适配浏览器主线程或 Web Worker。
#[wasm_bindgen]
pub struct WasmOfdReader {
    core: OfdReaderCore,
}

#[wasm_bindgen]
impl WasmOfdReader {
    /// 从 OFD 文件的原始字节构造读取器。
    ///
    /// # Errors
    ///
    /// 数据不是合法 OFD ZIP 包时返回 JS 异常。
    pub fn from_bytes(data: &[u8]) -> Result<WasmOfdReader, JsValue> {
        let core = OfdReaderCore::from_bytes(data).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self { core })
    }

    /// 文档总页数。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.core.page_count()
    }

    /// 获取指定页面的尺寸（宽 x 高，单位 mm）。
    ///
    /// # Errors
    ///
    /// 页码越界时返回 JS 异常。
    pub fn page_size(&self, index: usize) -> Result<Size, JsValue> {
        let (w, h) = self
            .core
            .page_size(index)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Size {
            width: w,
            height: h,
        })
    }

    /// 提取指定页面的全部文本块。
    ///
    /// 每个文本对象可能包含多行，按换行符拆分为独立字符串。
    /// 页码越界时返回空数组。wasm-bindgen 自动将 `Vec<String>` 转为 JS `string[]`。
    #[must_use]
    pub fn page_texts(&self, index: usize) -> Vec<String> {
        self.core.page_texts(index)
    }

    /// 文档元数据的 JSON 表示。
    ///
    /// 包含标题、作者、创建者、创建日期、修改日期、文档 ID、关键词、主题等字段。
    /// 缺失字段在 JSON 中为 `null`。
    #[must_use]
    pub fn metadata_json(&self) -> String {
        self.core.metadata_json()
    }

    /// 提取文档全部文本并以 Markdown 格式组织输出。
    ///
    /// 每页以 `## 第 N 页` 标题分隔，页面内文本按原始顺序逐行输出。
    /// 适合浏览器端快速预览 OFD 文本内容。
    #[must_use]
    pub fn to_markdown(&self) -> String {
        self.core.to_markdown()
    }

    /// 签名验证摘要。
    ///
    /// 通过检查 ZIP 包内是否存在签名相关条目来判断：
    /// - `"valid"`：签名数据完整（存在 Signs 目录和 Signature.xml）
    /// - `"unsigned"`：存在签名目录但结构不完整
    /// - `"no_signature"`：文档未签名
    /// - `"error"`：ZIP 解析失败
    ///
    /// 注意：WASM 环境不支持完整 SM2/SM3 密码学验证（`easyofd-signature` 的
    /// `verify_signature` 依赖 `std::fs::read`），此方法仅做结构层面的摘要判断。
    #[must_use]
    pub fn verify_signature_summary(&self) -> String {
        self.core.verify_signature_summary()
    }
}

/// 辅助：将 `Option<String>` 插入 JSON map，`None` 映射为 `null`。
fn insert_opt_string(
    map: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    val: Option<&String>,
) {
    map.insert(
        key.into(),
        val.map_or(serde_json::Value::Null, |s| {
            serde_json::Value::String(s.clone())
        }),
    );
}

// ── 单元测试（仅 host 目标，测试 OfdReaderCore 纯 Rust 逻辑） ──────────────

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};

    /// 辅助：从 OfdPage 列表构建 OFD 字节。
    fn build_ofd_bytes(pages: Vec<OfdPage>) -> Vec<u8> {
        let mut writer = easyofd_writer::OfdWriter::new();
        for page in pages {
            writer.add_page(page);
        }
        writer.build().expect("构建 OFD 字节失败")
    }

    #[test]
    fn from_bytes_and_page_count() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "WASM 测试"));
        let bytes = build_ofd_bytes(vec![page]);

        let core = OfdReaderCore::from_bytes(&bytes).expect("from_bytes 应成功");
        assert_eq!(core.page_count(), 1);
    }

    #[test]
    fn page_size_returns_correct_dimensions() {
        let page = OfdPage::new(297.0, 210.0);
        let bytes = build_ofd_bytes(vec![page]);

        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        let (w, h) = core.page_size(0).expect("page_size(0) 应成功");
        assert!((w - 297.0).abs() < f64::EPSILON);
        assert!((h - 210.0).abs() < f64::EPSILON);
    }

    #[test]
    fn page_size_out_of_bounds_returns_error() {
        let bytes = build_ofd_bytes(vec![]);
        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        assert!(core.page_size(0).is_err());
    }

    #[test]
    fn page_texts_extracts_all_text_blocks() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "第一行"));
        page.add_text(TextObject::new(10.0, 40.0, "第二行"));
        let bytes = build_ofd_bytes(vec![page]);

        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        let texts = core.page_texts(0);
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0], "第一行");
        assert_eq!(texts[1], "第二行");
    }

    #[test]
    fn page_texts_out_of_bounds_returns_empty() {
        let bytes = build_ofd_bytes(vec![]);
        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        assert!(core.page_texts(0).is_empty());
    }

    #[test]
    fn metadata_json_produces_valid_json() {
        let page = OfdPage::new(210.0, 297.0);
        let bytes = build_ofd_bytes(vec![page]);

        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        let json = core.metadata_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("应为合法 JSON");
        assert_eq!(parsed["version"], "1.0");
        // 标题为 null（未设置）
        assert!(parsed["title"].is_null());
    }

    #[test]
    fn to_markdown_extracts_text_content() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "Markdown 内容"));
        let bytes = build_ofd_bytes(vec![page]);

        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        let md = core.to_markdown();
        assert!(md.contains("## 第 1 页"));
        assert!(md.contains("Markdown 内容"));
    }

    #[test]
    fn to_markdown_multiple_pages() {
        let mut p1 = OfdPage::new(210.0, 297.0);
        p1.add_text(TextObject::new(10.0, 20.0, "首页"));
        let mut p2 = OfdPage::new(210.0, 297.0);
        p2.add_text(TextObject::new(10.0, 20.0, "尾页"));
        let bytes = build_ofd_bytes(vec![p1, p2]);

        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        let md = core.to_markdown();
        assert!(md.contains("## 第 1 页"));
        assert!(md.contains("首页"));
        assert!(md.contains("## 第 2 页"));
        assert!(md.contains("尾页"));
    }

    #[test]
    fn verify_signature_summary_no_signature() {
        let page = OfdPage::new(210.0, 297.0);
        let bytes = build_ofd_bytes(vec![page]);

        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        assert_eq!(core.verify_signature_summary(), "no_signature");
    }

    #[test]
    fn from_bytes_invalid_data_returns_error() {
        let result = OfdReaderCore::from_bytes(b"not an ofd file");
        assert!(result.is_err());
    }

    #[test]
    fn empty_document() {
        let bytes = build_ofd_bytes(vec![]);
        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        assert_eq!(core.page_count(), 0);
        assert_eq!(core.verify_signature_summary(), "no_signature");
        assert_eq!(core.to_markdown(), "");
    }

    #[test]
    fn multiline_text_split_into_entries() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "行一\n行二\n行三"));
        let bytes = build_ofd_bytes(vec![page]);

        let core = OfdReaderCore::from_bytes(&bytes).unwrap();
        let texts = core.page_texts(0);
        assert_eq!(texts.len(), 3);
        assert_eq!(texts[0], "行一");
        assert_eq!(texts[1], "行二");
        assert_eq!(texts[2], "行三");
    }
}
