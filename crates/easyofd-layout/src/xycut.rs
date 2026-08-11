//! XY-cut 页面分割算法。
//!
//! 给定一个 OFD 页面的所有内容对象，按空白投影将页面递归分割为子区域
//! （栏 / 表格 / 段落）。算法源自 Document Analysis 领域经典的 XY-recursive cut：
//!
//! 1. 沿水平方向投影，找到最宽的空白带，将页面横向切分。
//! 2. 对每个水平条带沿垂直方向投影，找到最宽的空白列，纵向切分。
//! 3. 递归执行，直到区域面积小于阈值或内容不足。
//!
//! **简化点**：当前实现仅做单次水平-垂直交替递归（不支持多级嵌套表格的
//! 最优分割），且空白检测基于对象外接矩形而非像素级投影。对于大多数
//! 中文文档的双栏 / 简单表格场景已足够。

use easyofd_core::{ContentObject, OfdPage};

/// XY-cut 分割选项。
#[derive(Debug, Clone, PartialEq)]
pub struct XyCutOptions {
    /// 最小空白间距（mm），低于此值的间隙不作为分割线。
    pub min_gap: f64,
    /// 最小区域面积 (width, height)（mm），低于此值不再递归分割。
    pub min_region: (f64, f64),
}

impl Default for XyCutOptions {
    fn default() -> Self {
        Self {
            min_gap: 3.0,
            min_region: (10.0, 5.0),
        }
    }
}

/// 页面分割后的区域。
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    /// 外接矩形 (x, y, w, h)，单位 mm。
    pub bbox: (f64, f64, f64, f64),
    /// 落入该区域的内容对象在 `page.content` 中的下标。
    pub content_indices: Vec<usize>,
}

/// XY-cut 页面分割入口。
///
/// 将页面中的所有内容对象按 XY-cut 递归分割为若干区域。
/// 如果页面无内容，返回空 `Vec`。
#[must_use]
pub fn xycut(page: &OfdPage, options: &XyCutOptions) -> Vec<Region> {
    let bboxes = extract_bboxes(page);
    if bboxes.is_empty() {
        return Vec::new();
    }
    let indices: Vec<usize> = (0..bboxes.len()).collect();
    let page_bbox = (0.0, 0.0, page.width, page.height);
    let mut regions = Vec::new();
    cut_recursive(&bboxes, &indices, page_bbox, options, &mut regions);
    regions
}

/// 内容对象的轴对齐外接矩形 (x, y, w, h)。
type BBox = (f64, f64, f64, f64);

/// 从页面内容提取外接矩形。
fn extract_bboxes(page: &OfdPage) -> Vec<BBox> {
    page.content
        .iter()
        .map(|obj| match obj {
            ContentObject::Text(t) => {
                let w = t
                    .width
                    .unwrap_or_else(|| estimate_text_width(t.text.chars().count(), t.size));
                let h = t.height.unwrap_or(t.size * 0.352_8);
                (t.x, t.y, w, h)
            }
            ContentObject::Image(i) => (i.x, i.y, i.width, i.height),
            ContentObject::Path(p) => {
                let h = p.stroke_width.max(0.5);
                (p.x, p.y, h, h)
            }
        })
        .collect()
}

/// 估算文本宽度（mm）。
fn estimate_text_width(char_count: usize, size_pt: f64) -> f64 {
    let chars = u32::try_from(char_count).unwrap_or(u32::MAX);
    f64::from(chars) * size_pt * 0.06
}

/// 递归 XY-cut 核心。
fn cut_recursive(
    bboxes: &[BBox],
    indices: &[usize],
    region: BBox,
    options: &XyCutOptions,
    out: &mut Vec<Region>,
) {
    let (rx, ry, rw, rh) = region;

    // 区域太小，停止递归。
    if rw < options.min_region.0 || rh < options.min_region.1 {
        out.push(Region {
            bbox: region,
            content_indices: indices.to_vec(),
        });
        return;
    }

    // 只剩一个对象，直接产出。
    if indices.len() <= 1 {
        out.push(Region {
            bbox: region,
            content_indices: indices.to_vec(),
        });
        return;
    }

    // 阶段 1：水平切割（沿 Y 方向投影，找空白行）。
    let h_slices = compute_slices(indices, bboxes, |b| b.1, |b| b.1 + b.3, ry, ry + rh);
    if let Some(best) = find_best_gap(&h_slices, options.min_gap) {
        let slices = split_at_gap(&h_slices, best);
        for slice in slices {
            let new_region = (rx, slice.start, rw, slice.end - slice.start);
            let slice_indices: Vec<usize> = indices
                .iter()
                .copied()
                .filter(|&i| {
                    let by = bboxes[i].1;
                    let bh = bboxes[i].3;
                    by + bh > slice.start && by < slice.end
                })
                .collect();
            if !slice_indices.is_empty() {
                // 阶段 2：对每个水平条带做垂直切割。
                vertical_cut(bboxes, &slice_indices, new_region, options, out);
            }
        }
        return;
    }

    // 无法水平切割，尝试直接垂直切割。
    vertical_cut(bboxes, indices, region, options, out);
}

/// 垂直切割：沿 X 方向投影，找空白列。
fn vertical_cut(
    bboxes: &[BBox],
    indices: &[usize],
    region: BBox,
    options: &XyCutOptions,
    out: &mut Vec<Region>,
) {
    let (rx, ry, rw, rh) = region;

    let v_slices = compute_slices(indices, bboxes, |b| b.0, |b| b.0 + b.2, rx, rx + rw);
    if let Some(best) = find_best_gap(&v_slices, options.min_gap) {
        let slices = split_at_gap(&v_slices, best);
        for slice in slices {
            let new_region = (slice.start, ry, slice.end - slice.start, rh);
            let slice_indices: Vec<usize> = indices
                .iter()
                .copied()
                .filter(|&i| {
                    let bx = bboxes[i].0;
                    let bw = bboxes[i].2;
                    bx + bw > slice.start && bx < slice.end
                })
                .collect();
            if !slice_indices.is_empty() {
                cut_recursive(bboxes, &slice_indices, new_region, options, out);
            }
        }
        return;
    }

    // 无法进一步分割，产出当前区域。
    out.push(Region {
        bbox: region,
        content_indices: indices.to_vec(),
    });
}

/// 投影切片：`(start, end)` 表示一个被内容占据的区间。
#[derive(Debug, Clone)]
struct Slice {
    start: f64,
    end: f64,
}

/// 将对象投影到一维轴并合并重叠区间，返回有序切片。
fn compute_slices(
    indices: &[usize],
    bboxes: &[BBox],
    get_pos: impl Fn(&BBox) -> f64,
    get_end: impl Fn(&BBox) -> f64,
    axis_min: f64,
    axis_max: f64,
) -> Vec<Slice> {
    let mut intervals: Vec<(f64, f64)> = indices
        .iter()
        .map(|&i| {
            let p = get_pos(&bboxes[i]).max(axis_min);
            let e = get_end(&bboxes[i]).min(axis_max);
            (p, e)
        })
        .filter(|(p, e)| e > p)
        .collect();
    if intervals.is_empty() {
        return Vec::new();
    }
    intervals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut slices = Vec::new();
    let mut cur_start = intervals[0].0;
    let mut cur_end = intervals[0].1;
    for (s, e) in intervals.iter().skip(1) {
        if *s <= cur_end {
            cur_end = cur_end.max(*e);
        } else {
            slices.push(Slice {
                start: cur_start,
                end: cur_end,
            });
            cur_start = *s;
            cur_end = *e;
        }
    }
    slices.push(Slice {
        start: cur_start,
        end: cur_end,
    });
    slices
}

/// 在有序切片之间找到最宽的空白间隙。
/// 返回该间隙的中心坐标（如果满足 `min_gap` 条件）。
fn find_best_gap(slices: &[Slice], min_gap: f64) -> Option<f64> {
    if slices.len() < 2 {
        return None;
    }
    let mut best_gap = 0.0_f64;
    let mut best_pos = None;
    for pair in slices.windows(2) {
        let gap = pair[1].start - pair[0].end;
        if gap >= min_gap && gap > best_gap {
            best_gap = gap;
            best_pos = Some(f64::midpoint(pair[0].end, pair[1].start));
        }
    }
    best_pos
}

/// 按切割位置将切片分为两组，返回新的子区间。
fn split_at_gap(slices: &[Slice], cut_pos: f64) -> Vec<Slice> {
    let mut result = Vec::new();
    // 切割位置左侧的合并区间。
    let left_end = slices
        .iter()
        .filter(|s| s.end <= cut_pos)
        .map(|s| s.end)
        .next_back();
    if let Some(end) = left_end {
        let start = slices
            .iter()
            .find(|s| s.end > s.start)
            .map_or(0.0, |s| s.start);
        result.push(Slice { start, end });
    }
    // 切割位置右侧的合并区间。
    let right_start = slices
        .iter()
        .filter(|s| s.start >= cut_pos)
        .map(|s| s.start)
        .next();
    if let Some(start) = right_start {
        let end = slices.last().map_or(start, |s| s.end);
        result.push(Slice { start, end });
    }
    // 如果两侧都不完整（切割在某个区间内部），退化为以切割点为界。
    if result.is_empty() && !slices.is_empty() {
        let first = slices.first().expect("slices 非空");
        let last = slices.last().expect("slices 非空");
        result.push(Slice {
            start: first.start,
            end: cut_pos,
        });
        result.push(Slice {
            start: cut_pos,
            end: last.end,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{ImageObject, TextObject};

    // --- 空页面 ---

    #[test]
    fn empty_page_returns_no_regions() {
        let page = OfdPage::new(210.0, 297.0);
        let regions = xycut(&page, &XyCutOptions::default());
        assert!(regions.is_empty());
    }

    // --- 单对象 ---

    #[test]
    fn single_object_yields_single_region() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 10.0, "唯一"));
        let regions = xycut(&page, &XyCutOptions::default());
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].content_indices, vec![0]);
    }

    // --- 双栏布局 ---

    #[test]
    fn two_column_layout_splits_into_two_regions() {
        let mut page = OfdPage::new(210.0, 297.0);
        // 左栏：多行文本
        for i in 0..5 {
            let y = 20.0 + f64::from(i) * 10.0;
            page.add_text(TextObject::new(10.0, y, "左栏文本内容填充"));
        }
        // 右栏：多行文本，中间留 30mm 空白
        for i in 0..5 {
            let y = 20.0 + f64::from(i) * 10.0;
            page.add_text(TextObject::new(130.0, y, "右栏文本内容填充"));
        }
        let options = XyCutOptions {
            min_gap: 5.0,
            min_region: (10.0, 5.0),
        };
        let regions = xycut(&page, &options);
        // 应至少分割为 2 个区域
        assert!(
            regions.len() >= 2,
            "expected >= 2 regions, got {}",
            regions.len()
        );
        // 所有内容下标都应被覆盖
        let mut all_indices: Vec<usize> = regions
            .iter()
            .flat_map(|r| &r.content_indices)
            .copied()
            .collect();
        all_indices.sort_unstable();
        assert_eq!(all_indices, (0..10).collect::<Vec<_>>());
    }

    // --- 包含图片的页面 ---

    #[test]
    fn page_with_image_and_text() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 10.0, "标题"));
        page.add_image(ImageObject::jpeg(10.0, 30.0, 50.0, 40.0, vec![0xFF]));
        page.add_text(TextObject::new(10.0, 80.0, "正文"));
        let regions = xycut(&page, &XyCutOptions::default());
        assert!(!regions.is_empty());
        // 三个对象都被覆盖
        let total: usize = regions.iter().map(|r| r.content_indices.len()).sum();
        assert_eq!(total, 3);
    }

    // --- min_gap 过大时不切割 ---

    #[test]
    fn large_min_gap_prevents_split() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 10.0, "A"));
        page.add_text(TextObject::new(150.0, 10.0, "B"));
        let options = XyCutOptions {
            min_gap: 500.0, // 超过页面宽度
            min_region: (1.0, 1.0),
        };
        let regions = xycut(&page, &options);
        // 不满足切割条件，应合并为 1 个区域
        assert_eq!(regions.len(), 1);
    }

    // --- min_region 过大时提前停止 ---

    #[test]
    fn large_min_region_limits_recursion() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 10.0, "X"));
        page.add_text(TextObject::new(100.0, 10.0, "Y"));
        let options = XyCutOptions {
            min_gap: 1.0,
            min_region: (300.0, 300.0), // 大于页面
        };
        let regions = xycut(&page, &options);
        // 区域太大，直接产出 1 个
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].content_indices.len(), 2);
    }

    // --- 默认选项 ---

    #[test]
    fn default_options_are_reasonable() {
        let opts = XyCutOptions::default();
        assert!(opts.min_gap > 0.0);
        assert!(opts.min_region.0 > 0.0);
        assert!(opts.min_region.1 > 0.0);
    }

    // --- Region 结构 ---

    #[test]
    fn region_bbox_format_is_xywh() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(5.0, 5.0, "test"));
        let regions = xycut(&page, &XyCutOptions::default());
        let region = &regions[0];
        let (bbox_x, bbox_y, bbox_w, bbox_h) = region.bbox;
        assert!(bbox_x >= 0.0 && bbox_y >= 0.0);
        assert!(bbox_w > 0.0 && bbox_h > 0.0);
    }
}
