//! 字形数据记录。
//!
//! 对应 Java: org.ofdrw.converter.font.GlyphData

/// 边界框。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// 最小 X。
    pub x_min: f32,
    /// 最小 Y。
    pub y_min: f32,
    /// 最大 X。
    pub x_max: f32,
    /// 最大 Y。
    pub y_max: f32,
}

impl BoundingBox {
    /// 创建边界框。
    pub fn new(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> Self {
        Self {
            x_min,
            y_min,
            x_max,
            y_max,
        }
    }

    /// 创建空边界框。
    pub fn zero() -> Self {
        Self {
            x_min: 0.0,
            y_min: 0.0,
            x_max: 0.0,
            y_max: 0.0,
        }
    }

    /// 宽度。
    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    /// 高度。
    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }
}

/// 字形数据。
///
/// 对应 Java `GlyphData`。描述 `glyf` 表中单个字形的边界框和轮廓信息。
///
/// 参考 Apache FontBox，遵循 OpenType `glyf` 表规范。
#[derive(Debug, Clone)]
pub struct GlyphData {
    /// 最小 X 坐标。
    x_min: i16,
    /// 最小 Y 坐标。
    y_min: i16,
    /// 最大 X 坐标。
    x_max: i16,
    /// 最大 Y 坐标。
    y_max: i16,
    /// 轮廓数量（>= 0 为简单字形，< 0 为复合字形）。
    number_of_contours: i16,
    /// 边界框。
    bounding_box: BoundingBox,
}

impl GlyphData {
    /// 创建空字形数据。
    pub fn new() -> Self {
        Self {
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
            number_of_contours: 0,
            bounding_box: BoundingBox::zero(),
        }
    }

    /// 从原始字段创建字形数据。
    pub fn with_fields(
        number_of_contours: i16,
        x_min: i16,
        y_min: i16,
        x_max: i16,
        y_max: i16,
    ) -> Self {
        Self {
            x_min,
            y_min,
            x_max,
            y_max,
            number_of_contours,
            bounding_box: BoundingBox::new(x_min as f32, y_min as f32, x_max as f32, y_max as f32),
        }
    }

    /// 返回边界框。
    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }

    /// 设置边界框。
    pub fn set_bounding_box(&mut self, bb: BoundingBox) {
        self.bounding_box = bb;
    }

    /// 返回轮廓数量。
    pub fn number_of_contours(&self) -> i16 {
        self.number_of_contours
    }

    /// 设置轮廓数量。
    pub fn set_number_of_contours(&mut self, n: i16) {
        self.number_of_contours = n;
    }

    /// 返回 xMin。
    pub fn x_min(&self) -> i16 {
        self.x_min
    }

    /// 返回 yMin。
    pub fn y_min(&self) -> i16 {
        self.y_min
    }

    /// 返回 xMax。
    pub fn x_max(&self) -> i16 {
        self.x_max
    }

    /// 返回 yMax。
    pub fn y_max(&self) -> i16 {
        self.y_max
    }
}

impl Default for GlyphData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let gd = GlyphData::new();
        assert_eq!(gd.number_of_contours(), 0);
        assert_eq!(gd.x_min(), 0);
    }

    #[test]
    fn test_with_fields() {
        let gd = GlyphData::with_fields(2, -100, -200, 500, 600);
        assert_eq!(gd.number_of_contours(), 2);
        assert_eq!(gd.x_min(), -100);
        assert_eq!(gd.x_max(), 500);
        let bb = gd.bounding_box();
        assert!((bb.width() - 600.0).abs() < f32::EPSILON);
        assert!((bb.height() - 800.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bounding_box() {
        let mut gd = GlyphData::new();
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 200.0);
        gd.set_bounding_box(bb);
        assert!((gd.bounding_box().width() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_number_of_contours() {
        let mut gd = GlyphData::new();
        gd.set_number_of_contours(-1);
        assert_eq!(gd.number_of_contours(), -1);
    }

    #[test]
    fn test_default() {
        let gd = GlyphData::default();
        assert_eq!(gd.number_of_contours(), 0);
    }
}
