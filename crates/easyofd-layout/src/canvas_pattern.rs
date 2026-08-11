//! Canvas 填充模式（底纹）。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.CanvasPattern

/// 底纹重复方式。
///
/// 对应 Java: ofdrw CanvasPattern 的 repetition 参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    /// 正常重复（默认，等同 repeat）。
    #[default]
    Normal,
    /// 列重复。
    Column,
    /// 行重复。
    Row,
    /// 行列重复。
    RowAndColumn,
}

impl RepeatMode {
    /// 从字符串解析重复方式（对应 Java: CanvasPattern 构造函数中的 switch）。
    ///
    /// 支持 `"repeat"` / `"normal"` / `"column"` / `"row"` / `"row-column"`。
    /// 不识别的值返回 `None`。
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "repeat" | "normal" | "" => Some(Self::Normal),
            "column" => Some(Self::Column),
            "row" => Some(Self::Row),
            "row-column" => Some(Self::RowAndColumn),
            _ => None,
        }
    }
}

/// Canvas 填充模式（底纹）。
///
/// 对应 Java: ofdrw layout canvas CanvasPattern。
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasPattern {
    /// 底纹图片路径标识。
    pub image_path: String,
    /// 重复方式。
    pub repeat_mode: RepeatMode,
    /// 底纹单元宽度（mm）。
    pub width: f64,
    /// 底纹单元高度（mm）。
    pub height: f64,
    /// 变换矩阵 `[a, b, c, d, e, f]`（可选）。
    pub transform: Option<[f64; 6]>,
}

impl CanvasPattern {
    /// 创建 Canvas 底纹（对应 Java: CanvasPattern(img, repetition, imgObj)）。
    #[must_use]
    pub fn new(
        image_path: impl Into<String>,
        repeat_mode: RepeatMode,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            image_path: image_path.into(),
            repeat_mode,
            width,
            height,
            transform: None,
        }
    }

    /// 设置底纹单元变换矩阵（对应 Java: CanvasPattern#setTransform(double[])）。
    ///
    /// 矩阵按 `[a, b, c, d, e, f]` 顺序 6 个参数。
    #[must_use]
    pub fn transform(mut self, matrix: [f64; 6]) -> Self {
        self.transform = Some(matrix);
        self
    }

    /// 设置图片尺寸（对应 Java: CanvasPattern#setImageSize）。
    #[must_use]
    pub fn image_size(mut self, w: f64, h: f64) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat_mode_parse() {
        assert_eq!(RepeatMode::parse("repeat"), Some(RepeatMode::Normal));
        assert_eq!(RepeatMode::parse(""), Some(RepeatMode::Normal));
        assert_eq!(RepeatMode::parse("column"), Some(RepeatMode::Column));
        assert_eq!(RepeatMode::parse("row"), Some(RepeatMode::Row));
        assert_eq!(
            RepeatMode::parse("row-column"),
            Some(RepeatMode::RowAndColumn)
        );
        assert_eq!(RepeatMode::parse("invalid"), None);
    }

    #[test]
    fn test_repeat_mode_case_insensitive() {
        assert_eq!(RepeatMode::parse("REPEAT"), Some(RepeatMode::Normal));
        assert_eq!(RepeatMode::parse("Column"), Some(RepeatMode::Column));
    }

    #[test]
    fn test_canvas_pattern_new() {
        let p = CanvasPattern::new("/path/to/img.png", RepeatMode::Normal, 10.0, 10.0);
        assert_eq!(p.image_path, "/path/to/img.png");
        assert_eq!(p.repeat_mode, RepeatMode::Normal);
        assert!((p.width - 10.0).abs() < f64::EPSILON);
        assert!(p.transform.is_none());
    }

    #[test]
    fn test_canvas_pattern_builders() {
        let p = CanvasPattern::new("img.png", RepeatMode::Row, 5.0, 5.0)
            .transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
            .image_size(20.0, 20.0);
        assert_eq!(p.transform, Some([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
        assert!((p.width - 20.0).abs() < f64::EPSILON);
        assert!((p.height - 20.0).abs() < f64::EPSILON);
    }
}
