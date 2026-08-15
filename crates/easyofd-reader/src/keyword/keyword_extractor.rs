//! 关键字抽取器。
//!
//! 对应 Java: org.ofdrw.reader.keyword.KeywordExtractor
//!
//! 支持两种搜索模式：
//! - [`KeywordExtractor::get_keyword_positions`]: 简化模式，基于 `OfdPage` 模型。
//! - [`KeywordExtractor::get_keyword_positions_from_text_codes`][]: 完整模式，
//!   支持跨 TextCode 边界的关键字定位，对齐 Java 版行为。
//!
//! ## 跨 TextCode 匹配算法
//!
//! 当关键字被 TextCode 边界切断时（如"电子印章"分属两个 TextCode），算法会
//! 拼接相邻 TextCode 的文本内容来定位完整关键字：
//!
//! 1. **普通匹配**：关键字完整包含在单个 TextCode 中。
//! 2. **前缀匹配**：当前 TextCode 的内容是关键字的前缀，向后拼接后续 TextCode。
//! 3. **后缀匹配**：当前 TextCode 的末尾匹配关键字的开头，向后拼接。
//!
//! 坐标计算使用 DeltaX/DeltaY 逐字符偏移，对齐 Java `POINT_PER_MM` 语义。

use super::KeywordPosition;
use easyofd_core::{OfdPage, ST_Box};

/// 每毫米的 point 单位（72pt / 25.4mm）。
const POINT_PER_MM: f64 = 72.0 / 25.4;

/// 带上下文的 TextCode 条目，用于跨 TextCode 边界的关键字搜索。
///
/// 对应 Java: `TextCode` + `KeywordResource`（`boundaryMapping` 条目）
///
/// 将 OFD 文字定位信息（TextCode）与所属文本对象属性（边界框、字号）绑定。
/// `content`、`x`、`y`、`delta_x`、`delta_y` 来自 OFD TextCode 元素；
/// `page`、`boundary`、`font_size` 来自父 TextObject 及其所属页面。
#[derive(Debug, Clone)]
pub struct TextCodeEntry {
    /// 文字内容。
    pub content: String,
    /// 文本起始 X 坐标（mm，相对于父 TextObject 边界）。None 表示继承前一个 TextCode 位置。
    pub x: Option<f64>,
    /// 文本起始 Y 坐标（mm）。None 表示继承前一个 TextCode 位置。
    pub y: Option<f64>,
    /// X 方向逐字符偏移（单位 mm，已展开压缩格式）。
    pub delta_x: Vec<f64>,
    /// Y 方向逐字符偏移（单位 mm，已展开压缩格式）。
    pub delta_y: Vec<f64>,
    /// 所在页码（从 1 开始）。
    pub page: usize,
    /// 父文本对象边界框（mm）。
    pub boundary: ST_Box,
    /// 字号（mm，对应 OFD CT_Text 的 Size 属性）。
    pub font_size: f64,
    /// 仿射变换矩阵（可选），对应 OFD CT_Text 的 CTM 属性。
    ///
    /// 6 元素 `[a, b, c, d, e, f]`，变换公式：
    /// - `x' = a * x + c * y + e`
    /// - `y' = b * x + d * y + f`
    ///
    /// 对应 Java: `CT_Text.getCTM()` + `KeywordExtractor#getCtmKeywordPosition`
    pub ctm: Option<[f64; 6]>,
}

impl TextCodeEntry {
    /// 创建新的 TextCode 条目。
    #[must_use]
    pub fn new(content: impl Into<String>, page: usize, boundary: ST_Box, font_size: f64) -> Self {
        Self {
            content: content.into(),
            x: None,
            y: None,
            delta_x: Vec::new(),
            delta_y: Vec::new(),
            page,
            boundary,
            font_size,
            ctm: None,
        }
    }

    /// 设置坐标。
    #[must_use]
    pub fn coordinate(mut self, x: f64, y: f64) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// 设置 X 方向偏移。
    #[must_use]
    pub fn delta_x(mut self, deltas: Vec<f64>) -> Self {
        self.delta_x = deltas;
        self
    }

    /// 设置 Y 方向偏移。
    #[must_use]
    pub fn delta_y(mut self, deltas: Vec<f64>) -> Self {
        self.delta_y = deltas;
        self
    }

    /// 设置仿射变换矩阵（CTM）。
    ///
    /// 对应 Java: `CT_Text.getCTM()` 在 `KeywordExtractor#getCtmKeywordPosition` 中的使用。
    ///
    /// 6 元素 `[a, b, c, d, e, f]`，变换公式：
    /// - `x' = a * x + c * y + e`
    /// - `y' = b * x + d * y + f`
    #[must_use]
    pub fn ctm(mut self, ctm: [f64; 6]) -> Self {
        self.ctm = Some(ctm);
        self
    }
}

// ── 辅助函数 ────────────────────────────────────────────────────────────────

/// 将字符索引转换为字节索引。
fn char_to_byte_offset(s: &str, char_offset: usize) -> usize {
    s.char_indices()
        .nth(char_offset)
        .map_or(s.len(), |(i, _)| i)
}

/// 确保 delta 数组长度至少为 `len`，不足时用末值补齐。
///
/// 对应 Java: `DeltaTool.getDelta(ST_Array, int)` 的补齐逻辑。
/// 若 delta 为空则返回空（表示无偏移信息）。
fn pad_delta(deltas: &[f64], content_len: usize) -> Vec<f64> {
    if deltas.is_empty() || content_len == 0 {
        return Vec::new();
    }
    if deltas.len() >= content_len {
        return deltas.to_vec();
    }
    let mut result = deltas.to_vec();
    let last = *result.last().unwrap();
    result.resize(content_len, last);
    result
}

/// 在字符切片中从 `from` 位置开始查找子序列。
///
/// 对应 Java: `String.indexOf(String, int)`
fn char_find_from(haystack: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || from > haystack.len() {
        return None;
    }
    let end = haystack.len().saturating_sub(needle.len());
    for i in from..=end {
        if haystack[i..].starts_with(needle) {
            return Some(i);
        }
    }
    None
}

/// 检查后缀匹配：content 的尾部是否为 keyword 的前缀。
///
/// 对应 Java: `KeywordExtractor#checkPostfixMatch`
///
/// 查找 content 中最后一个 keyword 首字符的位置，然后验证从该位置到
/// content 末尾的字符序列是否匹配 keyword 的开头。
fn check_postfix_match(content_chars: &[char], keyword_chars: &[char]) -> Option<usize> {
    if keyword_chars.is_empty() || content_chars.is_empty() {
        return None;
    }

    let first_char = keyword_chars[0];
    // 对应 Java: content.lastIndexOf(keyword.charAt(0))
    let start_index = content_chars.iter().rposition(|&c| c == first_char)?;

    // 对应 Java: for (j = startIndex, k = 0; j < content.length(); j++, k++)
    for (k, &ch) in content_chars[start_index..].iter().enumerate() {
        if k >= keyword_chars.len() || ch != keyword_chars[k] {
            return None;
        }
    }

    Some(start_index)
}

/// 检索后续 TextCode 条目，拼接文本直到匹配关键字或失配。
///
/// 对应 Java: `KeywordExtractor#searchNextText`
///
/// 从 `start_index + 1` 开始遍历同页条目，逐步拼接内容。当拼接结果
/// 等于或以关键字开头时停止（完全匹配）；当关键字以拼接结果开头时继续
/// （部分匹配）；否则失配停止。`merge_indices` 在调用前已包含起始条目索引。
fn search_next_text(
    entries: &[TextCodeEntry],
    start_index: usize,
    keyword: &str,
    first_match_content: &str,
) -> Vec<usize> {
    let mut merge_indices = vec![start_index];
    let mut merge_text = String::from(first_match_content);
    let current_page = entries[start_index].page;

    for j in (start_index + 1)..entries.len() {
        let next = &entries[j];
        // 对应 Java: "".equals(next.getContent().trim()) → continue
        if next.content.trim().is_empty() {
            continue;
        }
        // 对应 Java: currentPage != nextKr.getPage() → break
        if next.page != current_page {
            break;
        }

        merge_text.push_str(&next.content);

        // 对应 Java: mergeTextString.equals(keyword) || mergeTextString.startsWith(keyword)
        if merge_text == keyword || merge_text.starts_with(keyword) {
            merge_indices.push(j);
            break;
        }
        // 对应 Java: keyword.startsWith(mergeTextString)
        if keyword.starts_with(&merge_text) {
            merge_indices.push(j);
        } else {
            break;
        }
    }

    merge_indices
}

/// 计算从 boundary 左上角偏移后的基准坐标。
///
/// 对应 Java: `KeywordExtractor#getLeftBottomPos`
fn get_base_xy(
    entry: &TextCodeEntry,
    delta_x: &[f64],
    delta_y: &[f64],
    char_offset: usize,
) -> (f64, f64) {
    let mut x = entry.boundary.top_left_x + entry.x.unwrap_or(0.0);
    let mut y = entry.boundary.top_left_y + entry.y.unwrap_or(0.0);
    for i in 0..char_offset {
        if i < delta_x.len() {
            x += delta_x[i];
        }
        if i < delta_y.len() {
            y += delta_y[i];
        }
    }
    (x, y)
}

/// 获取文本子串宽度。
///
/// 对应 Java: `KeywordExtractor#getStringWidth`
///
/// 宽度 = 字号 + 从 `start_char` 起 `char_count - 1` 个 delta 偏移之和。
fn get_string_width(start_char: usize, char_count: usize, delta_x: &[f64], font_size: f64) -> f64 {
    if char_count == 0 {
        return 0.0;
    }
    let mut width = font_size;
    for i in start_char..(start_char + char_count - 1) {
        if i < delta_x.len() {
            width += delta_x[i];
        }
    }
    width
}

/// 合并多个边界框为一个包含所有框的最小外接矩形。
///
/// 对应 Java: `KeywordExtractor#mergeBox`
fn merge_boxes(boxes: &[ST_Box]) -> ST_Box {
    if boxes.is_empty() {
        return ST_Box::new(0.0, 0.0, 0.0, 0.0);
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for b in boxes {
        min_x = min_x.min(b.top_left_x);
        min_y = min_y.min(b.top_left_y);
        max_x = max_x.max(b.top_left_x + b.width);
        max_y = max_y.max(b.top_left_y + b.height);
    }

    ST_Box::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

/// 对坐标应用 CTM 仿射变换。
///
/// 对应 Java: `KeywordExtractor#transform`
///
/// 变换公式（OFD CTM 语义，行优先仿射矩阵 `[a b c d e f]`）：
/// - `x' = a * sx + c * sy + e`
/// - `y' = b * sx + d * sy + f`
///
/// 与 Java 版 `transform` 方法保持一致的乘法顺序。
fn ctm_transform(matrix: &[f64; 6], sx: f64, sy: f64) -> (f64, f64) {
    let x = matrix[0] * sx + matrix[2] * sy + matrix[4];
    let y = matrix[1] * sx + matrix[3] * sy + matrix[5];
    (x, y)
}

/// 合并多个坐标点为最小外接矩形。
///
/// 对应 Java: `KeywordExtractor#mergePos`
fn merge_positions(positions: &[(f64, f64)]) -> ST_Box {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for &(px, py) in positions {
        min_x = min_x.min(px);
        min_y = min_y.min(py);
        max_x = max_x.max(px);
        max_y = max_y.max(py);
    }

    ST_Box::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

/// 计算单个 TextCode 中关键字的边界框。
///
/// 对应 Java: `KeywordExtractor#getKeywordPosition` / `KeywordExtractor#getCtmKeywordPosition`
///
/// 从 TextCode 基准位置开始，沿 DeltaX/DeltaY 逐字符行走，记录关键字
/// 字符区间内的最小/最大坐标，最终生成包含所有关键字字符的外接矩形。
///
/// 当 [`TextCodeEntry::ctm`] 存在时，对关键字区域的四个角应用仿射变换，
/// 再合并为外接矩形——对应 Java 版 `getCtmKeywordPosition` 的行为。
fn compute_keyword_box(
    entry: &TextCodeEntry,
    content_chars: &[char],
    text_index: usize,
    keyword_len: usize,
) -> ST_Box {
    let base_x = entry.x.unwrap_or(0.0);
    let base_y = entry.y.unwrap_or(0.0);
    let font_size = entry.font_size;

    if keyword_len == 0 || content_chars.is_empty() || text_index >= content_chars.len() {
        return ST_Box::new(
            entry.boundary.top_left_x + base_x,
            entry.boundary.top_left_y + base_y - font_size,
            font_size,
            font_size,
        );
    }

    let delta_x = pad_delta(&entry.delta_x, content_chars.len());
    let delta_y = pad_delta(&entry.delta_y, content_chars.len());

    // ── CTM 分支：对应 Java `KeywordExtractor#getCtmKeywordPosition` ──
    if let Some(matrix) = entry.ctm {
        // 步骤 1：沿 Delta 行走到关键字起始位置（TextCode 局部坐标）
        let mut x = base_x;
        let mut y = base_y;
        for i in 0..text_index {
            if i < delta_x.len() {
                x += delta_x[i];
            }
            if i < delta_y.len() {
                y += delta_y[i];
            }
        }

        // 步骤 2：计算关键字宽度
        let string_width = get_string_width(text_index, keyword_len, &delta_x, font_size);
        // 高度：用 font_size 近似（Java 用 strHeight 即 AWT FontRenderContext，
        // Rust 无 AWT，用 font_size 作为合理近似）
        let height = font_size;

        // 步骤 3：对关键字区域的四个角应用 CTM 仿射变换
        // 对应 Java: transform(matrix, x, y - height) 等
        let left_top = ctm_transform(&matrix, x, y - height);
        let left_bottom = ctm_transform(&matrix, x, y);
        let right_top = ctm_transform(&matrix, x + string_width, y - height);
        let right_bottom = ctm_transform(&matrix, x + string_width, y);

        // 步骤 4：合并四个变换后的点为外接矩形
        let mut ctm_box = merge_positions(&[left_top, left_bottom, right_top, right_bottom]);

        // 步骤 5：偏移到 boundary 左上角（Java: ctmBox += ctText.getBoundary().getTopLeftPos()）
        ctm_box.top_left_x += entry.boundary.top_left_x;
        ctm_box.top_left_y += entry.boundary.top_left_y;

        return ctm_box;
    }

    // ── 非 CTM 分支：对应 Java `KeywordExtractor#getKeywordPosition` ──
    let mut x = entry.boundary.top_left_x + base_x;
    let mut y = entry.boundary.top_left_y + base_y;

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    let end = text_index + keyword_len;
    for i in 0..end {
        if i >= text_index {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
        if i < delta_x.len() {
            x += delta_x[i];
        }
        if i < delta_y.len() {
            y += delta_y[i];
        }
    }

    let w = max_x - min_x + font_size;
    let h = max_y - min_y + font_size;

    ST_Box::new(min_x, min_y - font_size, w, h)
}

/// 计算跨多个 TextCode 的关键字合并边界框。
///
/// 对应 Java: `KeywordExtractor#mergeKeywordPosition`
///
/// 对合并列表中的每个 TextCode 计算其贡献的字符区间和边界框，
/// 然后合并所有框为一个外接矩形。
///
/// 当 [`TextCodeEntry::ctm`] 存在时，对每个 TextCode 贡献的左下/右上角
/// 应用仿射变换再合并——对应 Java 版 CTM 分支。
fn compute_merged_box(
    entries: &[TextCodeEntry],
    merge_indices: &[usize],
    first_start_index: usize,
    keyword_len: usize,
) -> ST_Box {
    let mut boxes = Vec::new();
    let mut total_length = 0;

    for (idx, &entry_idx) in merge_indices.iter().enumerate() {
        let entry = &entries[entry_idx];
        let content_chars: Vec<char> = entry.content.chars().collect();
        let content_len = content_chars.len();
        let delta_x = pad_delta(&entry.delta_x, content_len);
        let delta_y = pad_delta(&entry.delta_y, content_len);

        // 对应 Java: 计算当前 TextCode 贡献的字符数和起始偏移
        let (text_length, start_char) = if idx == 0 && first_start_index > 0 {
            let tl = content_len.saturating_sub(first_start_index);
            total_length = tl;
            (tl, first_start_index)
        } else if total_length + content_len > keyword_len {
            (keyword_len - total_length, 0)
        } else {
            total_length += content_len;
            (content_len, 0)
        };

        // 对应 Java: getStringWidth
        let start_for_width = if idx == 0 && first_start_index > 0 {
            first_start_index
        } else {
            0
        };
        let mut width = get_string_width(start_for_width, text_length, &delta_x, entry.font_size);
        if width <= 0.0 {
            width = entry.font_size;
        }

        let height = entry.font_size;

        // ── CTM 分支：对应 Java `mergeKeywordPosition` 中 CTM 处理 ──
        if let Some(matrix) = entry.ctm {
            // TextCode 局部坐标
            let mut x = entry.x.unwrap_or(0.0);
            let mut y = entry.y.unwrap_or(0.0);
            if idx == 0 && first_start_index > 0 {
                for j in 0..first_start_index {
                    if j < delta_x.len() {
                        x += delta_x[j];
                    }
                    if j < delta_y.len() {
                        y += delta_y[j];
                    }
                }
            }
            let left_bottom = ctm_transform(&matrix, x, y);
            let right_top = ctm_transform(&matrix, x + width, y - height);

            let mut ctm_box = merge_positions(&[left_bottom, right_top]);
            ctm_box.top_left_x += entry.boundary.top_left_x;
            ctm_box.top_left_y += entry.boundary.top_left_y;
            boxes.push(ctm_box);
        } else {
            // ── 非 CTM 分支 ──
            let (base_x, base_y) = get_base_xy(entry, &delta_x, &delta_y, start_char);
            let box_ = ST_Box::new(base_x, base_y - height, width, height);
            boxes.push(box_);
        }
    }

    merge_boxes(&boxes)
}

// ── KeywordExtractor ────────────────────────────────────────────────────────

/// 关键字抽取器。
///
/// 对应 Java: `org.ofdrw.reader.keyword.KeywordExtractor`
///
/// 在 OFD 文档的文本内容中搜索关键字，返回匹配位置的页面和矩形区域。
///
/// 注意：Java 版使用 AWT `FontRenderContext` 计算精确的字符边界，
/// Rust 版使用简化的文本宽度估算。对于精确排版，需要集成外部字体引擎。
#[derive(Debug)]
pub struct KeywordExtractor;

impl KeywordExtractor {
    /// 获取关键字在文档中的位置列表。
    ///
    /// 对应 Java: `KeywordExtractor.getKeyWordPositionList(OFDReader, String)`
    ///
    /// 搜索所有页面的文本内容，返回包含关键字的矩形区域。
    ///
    /// # 参数
    ///
    /// - `pages`: 已解析的页面列表。
    /// - `keyword`: 要搜索的关键字。
    #[must_use]
    pub fn get_keyword_positions(pages: &[OfdPage], keyword: &str) -> Vec<KeywordPosition> {
        if keyword.is_empty() {
            return Vec::new();
        }

        let mut positions = Vec::new();
        for (page_idx, page) in pages.iter().enumerate() {
            let page_num = page_idx + 1;
            let page_positions = Self::search_page(page, page_num, keyword);
            positions.extend(page_positions);
        }
        positions
    }

    /// 在单个页面中搜索关键字。
    #[allow(clippy::cast_precision_loss)]
    fn search_page(page: &OfdPage, page_num: usize, keyword: &str) -> Vec<KeywordPosition> {
        use easyofd_core::ContentObject;

        let mut positions = Vec::new();
        for obj in &page.content {
            if let ContentObject::Text(text_obj) = obj {
                let text = &text_obj.text;
                // 查找所有匹配位置
                let mut start = 0;
                while let Some(idx) = text[start..].find(keyword) {
                    let match_start = start + idx;
                    // 估算关键字在文本中的位置
                    // 使用平均字符宽度近似（每个字符约 3mm 宽度作为默认值）
                    let char_width = 3.0;
                    let x = text_obj.x + (match_start as f64) * char_width;
                    let y = text_obj.y;
                    let kw_width = (keyword.len() as f64) * char_width;
                    let kw_height = text_obj.size / POINT_PER_MM;

                    let rect = easyofd_core::ST_Box::new(x, y, kw_width, kw_height);
                    positions.push(KeywordPosition::new(page_num, rect).with_keyword(keyword));
                    start = match_start + keyword.len();
                }
            }
        }
        positions
    }

    /// 获取关键字坐标列表（带字体度量的精确版本）。
    ///
    /// 对应 Java: `KeywordExtractor.getKeyWordPositionList(OFDReader, String)`
    ///
    /// 此方法接受字体大小映射表，用于更精确地计算字符宽度。
    ///
    /// # 参数
    ///
    /// - `pages`: 已解析的页面列表。
    /// - `keyword`: 要搜索的关键字。
    /// - `font_sizes`: 字体大小映射（字体 ID -> 字号，单位毫米）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_keyword_positions_with_fonts(
        pages: &[OfdPage],
        keyword: &str,
        font_sizes: &std::collections::HashMap<String, f64>,
    ) -> Vec<KeywordPosition> {
        if keyword.is_empty() {
            return Vec::new();
        }

        let mut positions = Vec::new();
        for (page_idx, page) in pages.iter().enumerate() {
            let page_num = page_idx + 1;
            for obj in &page.content {
                if let easyofd_core::ContentObject::Text(text_obj) = obj {
                    let text = &text_obj.text;
                    let char_width = font_sizes
                        .get(&text_obj.font)
                        .map_or(3.0, |size| size / POINT_PER_MM * 0.6);

                    let mut start = 0;
                    while let Some(idx) = text[start..].find(keyword) {
                        let match_start = start + idx;
                        let x = text_obj.x + (match_start as f64) * char_width;
                        let y = text_obj.y;
                        let kw_width = (keyword.len() as f64) * char_width;
                        let kw_height = text_obj.size / POINT_PER_MM;

                        let rect = easyofd_core::ST_Box::new(x, y, kw_width, kw_height);
                        positions.push(KeywordPosition::new(page_num, rect).with_keyword(keyword));
                        start = match_start + keyword.len();
                    }
                }
            }
        }
        positions
    }

    // ── 跨 TextCode 边界匹配 ────────────────────────────────────────────────

    /// 从 TextCode 条目列表中搜索关键字位置（支持跨 TextCode 边界匹配）。
    ///
    /// 对应 Java: `KeywordExtractor.getKeyWordPositionList(OFDReader, String[], int[])`
    ///
    /// 此方法接受按阅读顺序排列的 TextCode 条目列表。当关键字被 TextCode
    /// 边界切断时（如"电子印章"分属两个 TextCode），算法会拼接相邻 TextCode
    /// 的文本内容来定位完整关键字。
    ///
    /// # 参数
    ///
    /// - `entries`: 按阅读顺序排列的 TextCode 条目列表。
    /// - `keyword`: 要搜索的关键字。
    ///
    /// # 匹配模式
    ///
    /// 1. **普通匹配**：关键字完整包含在单个 TextCode 中。
    /// 2. **前缀匹配**：当前 TextCode 的内容是关键字的前缀，向后拼接。
    /// 3. **后缀匹配**：当前 TextCode 的末尾匹配关键字的开头，向后拼接。
    #[must_use]
    pub fn get_keyword_positions_from_text_codes(
        entries: &[TextCodeEntry],
        keyword: &str,
    ) -> Vec<KeywordPosition> {
        if keyword.is_empty() || entries.is_empty() {
            return Vec::new();
        }

        let keyword_chars: Vec<char> = keyword.chars().collect();
        let mut positions = Vec::new();

        for i in 0..entries.len() {
            let entry = &entries[i];
            // 对应 Java: content == null || "".equals(content.trim()) → skip
            if entry.content.trim().is_empty() {
                continue;
            }

            let content_chars: Vec<char> = entry.content.chars().collect();

            // 1. 普通匹配：关键字完整包含在当前 TextCode 中
            // 对应 Java: content.indexOf(keyword) != -1
            if let Some(text_index) = char_find_from(&content_chars, &keyword_chars, 0) {
                Self::add_normal_keyword_entry(
                    entry,
                    keyword,
                    &keyword_chars,
                    text_index,
                    &mut positions,
                );
                continue;
            }

            // 2. 前缀匹配：当前内容是关键字的前缀
            // 对应 Java: keyword.indexOf(content) == 0 && i != textCodeList.size() - 1
            if keyword_chars.starts_with(&content_chars) && i != entries.len() - 1 {
                Self::add_prefix_break(entries, i, keyword, &keyword_chars, &mut positions);
                continue;
            }

            // 3. 后缀匹配：当前内容末尾匹配关键字开头
            // 对应 Java: checkPostfixMatch(content, keyword) != -1
            if let Some(start_index) = check_postfix_match(&content_chars, &keyword_chars) {
                Self::add_postfix_break(
                    entries,
                    i,
                    start_index,
                    keyword,
                    &keyword_chars,
                    &mut positions,
                );
            }
        }

        positions
    }

    /// 处理普通匹配（关键字在单个 TextCode 内）。
    ///
    /// 对应 Java: `KeywordExtractor#addNormalKeyword`
    ///
    /// 查找当前 TextCode 中关键字的所有出现位置，为每个位置计算边界框。
    fn add_normal_keyword_entry(
        entry: &TextCodeEntry,
        keyword: &str,
        keyword_chars: &[char],
        first_text_index: usize,
        positions: &mut Vec<KeywordPosition>,
    ) {
        let content_chars: Vec<char> = entry.content.chars().collect();
        let mut text_index = first_text_index;

        loop {
            let rect = compute_keyword_box(entry, &content_chars, text_index, keyword_chars.len());
            positions.push(KeywordPosition::new(entry.page, rect).with_keyword(keyword));

            // 对应 Java: textIndex = content.indexOf(keyword, textIndex + keywordLength)
            let next_start = text_index + keyword_chars.len();
            match char_find_from(&content_chars, keyword_chars, next_start) {
                Some(next_idx) => text_index = next_idx,
                None => break,
            }
        }
    }

    /// 处理前缀匹配（当前内容是关键字前缀，需要向后拼接）。
    ///
    /// 对应 Java: `KeywordExtractor#addPrefixBreakTextCodeList`
    fn add_prefix_break(
        entries: &[TextCodeEntry],
        start_index: usize,
        keyword: &str,
        keyword_chars: &[char],
        positions: &mut Vec<KeywordPosition>,
    ) {
        let first_content = entries[start_index].content.clone();
        let merge_indices = search_next_text(entries, start_index, keyword, &first_content);

        // 对应 Java: 拼接所有合并的 TextCode 内容，检查是否包含关键字
        let merged: String = merge_indices
            .iter()
            .map(|&idx| entries[idx].content.as_str())
            .collect();

        if merged.contains(keyword) {
            let page = entries[start_index].page;
            let rect = compute_merged_box(entries, &merge_indices, 0, keyword_chars.len());
            positions.push(KeywordPosition::new(page, rect).with_keyword(keyword));
        }
    }

    /// 处理后缀匹配（当前内容末尾匹配关键字开头）。
    ///
    /// 对应 Java: `KeywordExtractor#addPostfixBreakTextCodeList`
    fn add_postfix_break(
        entries: &[TextCodeEntry],
        start_index: usize,
        postfix_start: usize,
        keyword: &str,
        keyword_chars: &[char],
        positions: &mut Vec<KeywordPosition>,
    ) {
        let first_content = &entries[start_index].content;
        // 对应 Java: textCode.getContent().substring(startIndex)
        let byte_start = char_to_byte_offset(first_content, postfix_start);
        let first_match = &first_content[byte_start..];

        let merge_indices = search_next_text(entries, start_index, keyword, first_match);

        // 对应 Java: 拼接所有合并的 TextCode 的完整内容
        let merged: String = merge_indices
            .iter()
            .map(|&idx| entries[idx].content.as_str())
            .collect();

        if merged.contains(keyword) {
            let page = entries[start_index].page;
            let rect =
                compute_merged_box(entries, &merge_indices, postfix_start, keyword_chars.len());
            positions.push(KeywordPosition::new(page, rect).with_keyword(keyword));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::TextObject;

    fn make_page_with_text(text: &str) -> OfdPage {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, text).size(10.0));
        page
    }

    // ── 现有测试（回归验证） ─────────────────────────────────────────────────

    #[test]
    fn test_keyword_extractor_single_match() {
        let pages = vec![make_page_with_text("Hello World OFD Test")];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "OFD");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].page, 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("OFD"));
    }

    #[test]
    fn test_keyword_extractor_multiple_matches() {
        let pages = vec![make_page_with_text("OFD is an OFD format")];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "OFD");
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn test_keyword_extractor_no_match() {
        let pages = vec![make_page_with_text("Hello World")];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "OFD");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_keyword_extractor_empty_keyword() {
        let pages = vec![make_page_with_text("Hello")];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_keyword_extractor_multiple_pages() {
        let pages = vec![
            make_page_with_text("Page 1"),
            make_page_with_text("Page 2 OFD"),
        ];
        let positions = KeywordExtractor::get_keyword_positions(&pages, "OFD");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].page, 2);
    }

    #[test]
    fn test_keyword_extractor_with_fonts() {
        let pages = vec![make_page_with_text("Test OFD keyword")];
        let mut font_sizes = std::collections::HashMap::new();
        font_sizes.insert("SimHei".to_string(), 12.0);
        let positions =
            KeywordExtractor::get_keyword_positions_with_fonts(&pages, "OFD", &font_sizes);
        assert_eq!(positions.len(), 1);
    }

    #[test]
    fn test_point_per_mm_constant() {
        assert!((POINT_PER_MM - 2.8346).abs() < 0.01);
    }

    // ── 跨 TextCode 测试 ────────────────────────────────────────────────────

    /// 辅助函数：创建简单的 TextCode 条目。
    fn make_entry(content: &str, page: usize, x: f64, y: f64, font_size: f64) -> TextCodeEntry {
        TextCodeEntry::new(
            content,
            page,
            ST_Box::new(0.0, 0.0, 210.0, 297.0),
            font_size,
        )
        .coordinate(x, y)
    }

    #[test]
    fn test_single_text_code_match() {
        // 单 TextCode 内普通匹配（对齐简化模式行为）
        let entries = vec![make_entry("Hello World OFD Test", 1, 10.0, 20.0, 3.0)];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "OFD");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].page, 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("OFD"));
    }

    #[test]
    fn test_cross_2_text_codes() {
        // 关键字 "电子印章" 跨两个 TextCode: "电子" + "印章"
        let entries = vec![
            make_entry("电子", 1, 10.0, 20.0, 3.0),
            make_entry("印章", 1, 30.0, 20.0, 3.0),
        ];
        let positions =
            KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子印章");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].page, 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("电子印章"));
        // 合并框应跨越两个 TextCode
        assert!(positions[0].rect.width > 3.0);
    }

    #[test]
    fn test_cross_3_text_codes() {
        // 关键字 "中华人民共和国" 跨三个 TextCode
        let entries = vec![
            make_entry("中华", 1, 10.0, 20.0, 3.0),
            make_entry("人民", 1, 30.0, 20.0, 3.0),
            make_entry("共和国", 1, 50.0, 20.0, 3.0),
        ];
        let positions =
            KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "中华人民共和国");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("中华人民共和国"));
    }

    #[test]
    fn test_with_delta_x() {
        // DeltaX 用 g 压缩语法展开后的 TextCode（每个字符偏移 10mm）
        let entries = vec![
            TextCodeEntry::new("电子印章", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(10.0, 20.0)
                .delta_x(vec![10.0, 10.0, 10.0]),
        ];
        let positions =
            KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子印章");
        assert_eq!(positions.len(), 1);
        // 验证宽度计算：font_size + sum(delta_x[0..3]) = 3 + 30 = 33
        assert!((positions[0].rect.width - 33.0).abs() < 0.01);
    }

    #[test]
    fn test_with_delta_x_cross_text_codes() {
        // 跨 TextCode 带 DeltaX
        let entries = vec![
            TextCodeEntry::new("电子", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(10.0, 20.0)
                .delta_x(vec![10.0, 10.0]),
            TextCodeEntry::new("印章", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(30.0, 20.0)
                .delta_x(vec![10.0, 10.0]),
        ];
        let positions =
            KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子印章");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("电子印章"));
        // 合并框宽度 > 单个 TextCode 宽度
        assert!(positions[0].rect.width > 10.0);
    }

    #[test]
    fn test_no_match() {
        let entries = vec![make_entry("Hello World", 1, 10.0, 20.0, 3.0)];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_empty_keyword_from_entries() {
        let entries = vec![make_entry("Hello", 1, 10.0, 20.0, 3.0)];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_empty_entries() {
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&[], "OFD");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_postfix_match() {
        // "abc电" 末尾的 "电" 是 "电子" 的前缀
        let entries = vec![
            make_entry("abc电", 1, 10.0, 20.0, 3.0),
            make_entry("子印章", 1, 40.0, 20.0, 3.0),
        ];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("电子"));
    }

    #[test]
    fn test_multiple_matches_same_entry() {
        let entries = vec![make_entry("OFD是OFD格式", 1, 10.0, 20.0, 3.0)];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "OFD");
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn test_cross_page_boundary_no_match() {
        // 不同页的 TextCode 不应跨页匹配
        let entries = vec![
            make_entry("电子", 1, 10.0, 20.0, 3.0),
            make_entry("印章", 2, 10.0, 20.0, 3.0),
        ];
        let positions =
            KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子印章");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_skip_empty_content() {
        // 空白 TextCode 应被跳过，不影响匹配
        let entries = vec![
            make_entry("电子", 1, 10.0, 20.0, 3.0),
            make_entry("  ", 1, 20.0, 20.0, 3.0),
            make_entry("印章", 1, 30.0, 20.0, 3.0),
        ];
        let positions =
            KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子印章");
        assert_eq!(positions.len(), 1);
    }

    #[test]
    fn test_postfix_with_delta() {
        // 后缀匹配 + DeltaX 验证坐标计算
        let entries = vec![
            TextCodeEntry::new("x电", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(5.0, 20.0)
                .delta_x(vec![8.0, 8.0]),
            TextCodeEntry::new("子", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(21.0, 20.0),
        ];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("电子"));
        // 验证合并框起始 X 坐标应包含 delta 偏移
        // 第一段起始: boundary.x + x + delta_x[0] = 0 + 5 + 8 = 13
        assert!((positions[0].rect.top_left_x - 13.0).abs() < 0.01);
    }

    #[test]
    fn test_keyword_not_in_any_entry() {
        let entries = vec![
            make_entry("abc", 1, 10.0, 20.0, 3.0),
            make_entry("def", 1, 30.0, 20.0, 3.0),
        ];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "xyz");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_partial_prefix_no_completion() {
        // 前缀匹配但后续 TextCode 无法凑齐关键字
        let entries = vec![
            make_entry("电", 1, 10.0, 20.0, 3.0),
            make_entry("xxx", 1, 30.0, 20.0, 3.0),
        ];
        let positions =
            KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子印章");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_position_coordinates_basic() {
        // 验证单 TextCode 匹配的坐标计算
        let entries = vec![
            TextCodeEntry::new("AB", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(10.0, 20.0)
                .delta_x(vec![10.0, 10.0]),
        ];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "AB");
        assert_eq!(positions.len(), 1);
        // 起始位置: boundary(0,0) + textcode(10,20) = (10, 20)
        // A 位置: x=10
        // B 位置: x=10+10=20 (delta_x[0]=10)
        // minX=10, maxX=20, width = 20-10+3 = 13
        assert!((positions[0].rect.width - 13.0).abs() < 0.01);
        // height = maxY-minY+font_size = 0+3 = 3
        assert!((positions[0].rect.height - 3.0).abs() < 0.01);
        // top_left_x = minX = 10
        assert!((positions[0].rect.top_left_x - 10.0).abs() < 0.01);
        // top_left_y = minY - font_size = 20 - 3 = 17
        assert!((positions[0].rect.top_left_y - 17.0).abs() < 0.01);
    }

    // ── CTM 仿射变换测试 ────────────────────────────────────────────────────

    #[test]
    fn test_ctm_transform_identity() {
        // 单位矩阵：变换后坐标不变
        let matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let (x, y) = ctm_transform(&matrix, 10.0, 20.0);
        assert!((x - 10.0).abs() < f64::EPSILON);
        assert!((y - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ctm_transform_translation() {
        // 平移矩阵 [1 0 0 1 5 10]：x'=x+5, y'=y+10
        let matrix = [1.0, 0.0, 0.0, 1.0, 5.0, 10.0];
        let (x, y) = ctm_transform(&matrix, 0.0, 0.0);
        assert!((x - 5.0).abs() < f64::EPSILON);
        assert!((y - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ctm_transform_90_rotation() {
        // 90 度逆时针旋转矩阵 [0 1 -1 0 0 0]
        // x' = 0*x + (-1)*y + 0 = -y
        // y' = 1*x + 0*y + 0 = x
        let matrix = [0.0, 1.0, -1.0, 0.0, 0.0, 0.0];
        let (x, y) = ctm_transform(&matrix, 3.0, 4.0);
        assert!((x - (-4.0)).abs() < f64::EPSILON);
        assert!((y - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ctm_keyword_single_match() {
        // 对应 Java: KeywordExtractor#getCtmKeywordPosition
        // 单 TextCode 带 CTM（单位矩阵），结果应与无 CTM 一致
        let entries = vec![
            TextCodeEntry::new("OFD", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(10.0, 20.0)
                .ctm([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        ];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "OFD");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("OFD"));
    }

    #[test]
    fn test_ctm_keyword_90_rotation() {
        // 90 度旋转文本，关键字 "AB" 的边界框应反映旋转变换
        // CTM = [0 1 -1 0 0 0]（90 度逆时针）
        // 原始: x=10, y=20, width=font_size(3.0), height=font_size(3.0)
        // 变换后:
        //   leftTop(10, 20-3)    -> (-(17), 10) = (-17, 10)
        //   leftBottom(10, 20)   -> (-20, 10)
        //   rightTop(13, 20-3)   -> (-17, 13)
        //   rightBottom(13, 20)  -> (-20, 13)
        // 合并: minX=-20, minY=10, maxX=-17, maxY=13
        // 最终: (-20, 10, 3, 3)
        let entries = vec![
            TextCodeEntry::new("AB", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(10.0, 20.0)
                .ctm([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]),
        ];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "AB");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("AB"));
        // 旋转后框尺寸: width=3, height=3
        assert!((positions[0].rect.width - 3.0).abs() < 0.01);
        assert!((positions[0].rect.height - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_ctm_keyword_with_translation() {
        // CTM 平移 [1 0 0 1 50 100]：所有坐标偏移 (+50, +100)
        // 原始: x=10, y=20, boundary(0,0)
        // 非 CTM 结果: top_left = (10, 20-3) = (10, 17)
        // CTM 结果: transform(10, 20) = (60, 120), transform(10, 17) = (60, 117)
        //   加 boundary 偏移后: (60, 117)
        let entries = vec![
            TextCodeEntry::new("AB", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(10.0, 20.0)
                .ctm([1.0, 0.0, 0.0, 1.0, 50.0, 100.0]),
        ];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "AB");
        assert_eq!(positions.len(), 1);
        // 验证平移效果：框整体偏移了 (50, 100)
        assert!((positions[0].rect.top_left_x - 60.0).abs() < 0.01);
        assert!((positions[0].rect.top_left_y - 117.0).abs() < 0.01);
    }

    #[test]
    fn test_ctm_keyword_cross_text_codes() {
        // 跨 TextCode 带 CTM（单位矩阵），验证合并框正常工作
        let entries = vec![
            TextCodeEntry::new("电", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(10.0, 20.0)
                .ctm([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            TextCodeEntry::new("子", 1, ST_Box::new(0.0, 0.0, 210.0, 297.0), 3.0)
                .coordinate(20.0, 20.0)
                .ctm([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        ];
        let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].keyword.as_deref(), Some("电子"));
        // 单位矩阵下，合并框应与无 CTM 行为一致
        assert!(positions[0].rect.width > 3.0);
    }
}
