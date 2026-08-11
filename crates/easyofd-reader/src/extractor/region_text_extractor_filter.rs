//! 矩形框文本提取过滤器。
//!
//! 对应 Java: org.ofdrw.reader.extractor.RegionTextExtractorFilter
//!
//! Java 版使用 `java.awt.Rectangle` 进行精确的字符级区域过滤，
//! 需要配合 DeltaTool 和 CTM 变换矩阵。Rust 版提供简化的区域过滤逻辑。

use easyofd_core::ST_Box;

/// 矩形框文本提取过滤器。
///
/// 对应 Java: `org.ofdrw.reader.extractor.RegionTextExtractorFilter`
///
/// 只提取指定矩形区域内的文本字符。Java 版对每个字符逐一判断
/// 是否落在矩形框内（考虑 CTM 变换和 DeltaX/DeltaY 偏移），
/// Rust 版基于文本对象的 Boundary 属性做简化判断。
#[derive(Debug, Clone)]
pub struct RegionTextExtractorFilter {
    /// 目标提取区域。
    region: ST_Box,
}

impl RegionTextExtractorFilter {
    /// 创建新的矩形框文本提取过滤器。
    ///
    /// # 参数
    ///
    /// - `region`: 目标提取区域（毫米，格式：左上角 x, y, 宽, 高）。
    #[must_use]
    pub fn new(region: ST_Box) -> Self {
        Self { region }
    }

    /// 判断文本对象的 Boundary 是否与目标区域有交集。
    ///
    /// 简化实现：判断两个矩形是否有重叠区域。
    #[must_use]
    pub fn intersects(&self, text_boundary: &ST_Box) -> bool {
        let self_right = self.region.top_left_x + self.region.width;
        let self_bottom = self.region.top_left_y + self.region.height;
        let other_right = text_boundary.top_left_x + text_boundary.width;
        let other_bottom = text_boundary.top_left_y + text_boundary.height;

        // 两个矩形不相交的条件取反
        !(text_boundary.top_left_x > self_right
            || other_right < self.region.top_left_x
            || text_boundary.top_left_y > self_bottom
            || other_bottom < self.region.top_left_y)
    }

    /// 判断文本对象是否完全包含在目标区域内。
    #[must_use]
    pub fn contains(&self, text_boundary: &ST_Box) -> bool {
        let self_right = self.region.top_left_x + self.region.width;
        let self_bottom = self.region.top_left_y + self.region.height;
        let other_right = text_boundary.top_left_x + text_boundary.width;
        let other_bottom = text_boundary.top_left_y + text_boundary.height;

        text_boundary.top_left_x >= self.region.top_left_x
            && text_boundary.top_left_y >= self.region.top_left_y
            && other_right <= self_right
            && other_bottom <= self_bottom
    }

    /// 获取目标提取区域。
    #[must_use]
    pub fn region(&self) -> &ST_Box {
        &self.region
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersects_overlapping() {
        let filter = RegionTextExtractorFilter::new(ST_Box::new(0.0, 0.0, 100.0, 100.0));
        let text_box = ST_Box::new(50.0, 50.0, 100.0, 100.0);
        assert!(filter.intersects(&text_box));
    }

    #[test]
    fn test_intersects_no_overlap() {
        let filter = RegionTextExtractorFilter::new(ST_Box::new(0.0, 0.0, 50.0, 50.0));
        let text_box = ST_Box::new(60.0, 60.0, 50.0, 50.0);
        assert!(!filter.intersects(&text_box));
    }

    #[test]
    fn test_contains_inside() {
        let filter = RegionTextExtractorFilter::new(ST_Box::new(0.0, 0.0, 100.0, 100.0));
        let text_box = ST_Box::new(10.0, 10.0, 20.0, 20.0);
        assert!(filter.contains(&text_box));
    }

    #[test]
    fn test_contains_partial() {
        let filter = RegionTextExtractorFilter::new(ST_Box::new(0.0, 0.0, 50.0, 50.0));
        let text_box = ST_Box::new(40.0, 40.0, 20.0, 20.0);
        assert!(!filter.contains(&text_box));
        assert!(filter.intersects(&text_box));
    }

    #[test]
    fn test_region_accessor() {
        let region = ST_Box::new(10.0, 20.0, 30.0, 40.0);
        let filter = RegionTextExtractorFilter::new(region.clone());
        assert!((filter.region().top_left_x - 10.0).abs() < f64::EPSILON);
    }
}
