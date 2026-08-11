//! 布局图片元素。
//!
//! 对应 Java: org.ofdrw.layout.element.Img

use std::path::PathBuf;

/// 布局图片元素（ofdrw layout Img）。
///
/// 对应 Java: ofdrw Img（Div 的图片变体），从文件加载图片并指定尺寸。
#[derive(Debug, Clone, PartialEq)]
pub struct Img {
    /// 图片源文件路径。
    pub src: PathBuf,
    /// 显示宽度（mm）。
    pub width: f64,
    /// 显示高度（mm）。
    pub height: f64,
}

impl Img {
    /// 创建图片元素（对应 Java: Img(width, height, src)）。
    #[must_use]
    pub fn new(width: f64, height: f64, src: PathBuf) -> Self {
        Self { src, width, height }
    }

    /// 从文件路径创建（对应 Java: Img(src)，尺寸由调用方设置）。
    #[must_use]
    pub fn from_path(src: PathBuf) -> Self {
        Self {
            src,
            width: 0.0,
            height: 0.0,
        }
    }

    /// 设置图片源（对应 Java: Img#setSrc）。
    #[must_use]
    pub fn src(mut self, src: PathBuf) -> Self {
        self.src = src;
        self
    }

    /// 设置显示宽度。
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// 设置显示高度。
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// 文件扩展名（用于判断图片格式）。
    #[must_use]
    pub fn extension(&self) -> Option<String> {
        self.src
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_img_new() {
        let img = Img::new(100.0, 50.0, PathBuf::from("/tmp/pic.png"));
        assert!((img.width - 100.0).abs() < f64::EPSILON);
        assert!((img.height - 50.0).abs() < f64::EPSILON);
        assert_eq!(img.extension().as_deref(), Some("png"));
    }

    #[test]
    fn test_builders() {
        let img = Img::from_path(PathBuf::from("/tmp/a.jpg"))
            .width(200.0)
            .height(100.0);
        assert!((img.width - 200.0).abs() < f64::EPSILON);
        assert_eq!(img.extension().as_deref(), Some("jpg"));
    }
}
