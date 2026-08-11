//! 文档缩放模式。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.doc.vpreferences.zoom.ZoomMode

/// 文档查看缩放模式（ofd:ZoomMode）。
///
/// 对应 Java: ofdrw ZoomMode.Type。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZoomMode {
    /// 默认缩放。
    #[default]
    Default,
    /// 适应高度。
    FitHeight,
    /// 适应宽度。
    FitWidth,
    /// 适应矩形区域。
    FitRect,
}

impl ZoomMode {
    /// 解析为 XML 文本（对应 Java: ZoomMode#toString）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::FitHeight => "FitHeight",
            Self::FitWidth => "FitWidth",
            Self::FitRect => "FitRect",
        }
    }
}

impl std::fmt::Display for ZoomMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(ZoomMode::default(), ZoomMode::Default);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(ZoomMode::FitHeight.as_str(), "FitHeight");
        assert_eq!(ZoomMode::FitRect.to_string(), "FitRect");
    }
}
