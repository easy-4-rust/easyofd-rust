//! 视图首选项（CT_VPreferences）。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.doc.vpreferences.CT_VPreferences
//!
//! 本标准支持设置文档视图首选项（VPreferences）节点，
//! 以达到限定文档初始化视图便于阅读的目的。
//! GB/T 33190 第 7.5 节 图 10。

/// 窗口模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageMode {
    /// 不使用特殊模式（默认值）。
    None,
    /// 全屏模式。
    FullScreen,
    /// 显示大纲。
    ShowOutline,
    /// 显示附件。
    ShowAttachments,
}

impl PageMode {
    /// 获取枚举的字符串表示。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::FullScreen => "FullScreen",
            Self::ShowOutline => "ShowOutline",
            Self::ShowAttachments => "ShowAttachments",
        }
    }

    /// 从字符串解析。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "None" => Ok(Self::None),
            "FullScreen" => Ok(Self::FullScreen),
            "ShowOutline" => Ok(Self::ShowOutline),
            "ShowAttachments" => Ok(Self::ShowAttachments),
            _ => Err(format!("未知的窗口模式: {s}")),
        }
    }
}

impl std::fmt::Display for PageMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 页面布局模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLayout {
    /// 单页显示（默认值）。
    OneColumn,
    /// 单页连续滚动。
    OneColumnContinuous,
    /// 双页对开。
    TwoPageLeft,
    /// 双页连续。
    TwoPageRight,
}

impl PageLayout {
    /// 获取枚举的字符串表示。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneColumn => "OneColumn",
            Self::OneColumnContinuous => "OneColumnContinuous",
            Self::TwoPageLeft => "TwoPageLeft",
            Self::TwoPageRight => "TwoPageRight",
        }
    }

    /// 从字符串解析。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "OneColumn" => Ok(Self::OneColumn),
            "OneColumnContinuous" => Ok(Self::OneColumnContinuous),
            "TwoPageLeft" => Ok(Self::TwoPageLeft),
            "TwoPageRight" => Ok(Self::TwoPageRight),
            _ => Err(format!("未知的页面布局: {s}")),
        }
    }
}

impl std::fmt::Display for PageLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 标题栏显示模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabDisplay {
    /// 显示文件名（默认值）。
    FileName,
    /// 显示文档标题。
    DocTitle,
}

impl TabDisplay {
    /// 获取枚举的字符串表示。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FileName => "FileName",
            Self::DocTitle => "DocTitle",
        }
    }

    /// 从字符串解析。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "FileName" => Ok(Self::FileName),
            "DocTitle" => Ok(Self::DocTitle),
            _ => Err(format!("未知的标题栏显示模式: {s}")),
        }
    }
}

impl std::fmt::Display for TabDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 缩放模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomMode {
    /// 适应页面。
    FitPage,
    /// 适应宽度。
    FitWidth,
    /// 适应高度。
    FitHeight,
}

impl ZoomMode {
    /// 获取枚举的字符串表示。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FitPage => "FitPage",
            Self::FitWidth => "FitWidth",
            Self::FitHeight => "FitHeight",
        }
    }
}

impl std::fmt::Display for ZoomMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 缩放设置（模式或具体数值）。
#[derive(Debug, Clone, Copy)]
pub enum ZoomScale {
    /// 按模式缩放。
    Mode(ZoomMode),
    /// 按具体比例缩放（百分比，如 100.0 表示 100%）。
    Value(f64),
}

/// 视图首选项。
///
/// 定义文档初始化视图的窗口模式、页面布局、
/// 工具栏可见性、缩放等设置。
#[derive(Debug, Clone)]
pub struct CtVPreferences {
    /// 窗口模式（可选），默认 None。
    pub page_mode: PageMode,
    /// 页面布局模式（可选），默认 OneColumn。
    pub page_layout: PageLayout,
    /// 标题栏显示模式（可选），默认 FileName。
    pub tab_display: Option<TabDisplay>,
    /// 是否隐藏工具栏（可选），默认 false。
    pub hide_toolbar: bool,
    /// 是否隐藏菜单栏（可选），默认 false。
    pub hide_menubar: bool,
    /// 是否隐藏主窗口之外的其他窗口组件（可选），默认 false。
    pub hide_window_ui: bool,
    /// 缩放设置（可选）。
    pub zoom: Option<ZoomScale>,
}

impl CtVPreferences {
    /// 创建默认的视图首选项。
    #[must_use]
    pub fn new() -> Self {
        Self {
            page_mode: PageMode::None,
            page_layout: PageLayout::OneColumn,
            tab_display: None,
            hide_toolbar: false,
            hide_menubar: false,
            hide_window_ui: false,
            zoom: None,
        }
    }

    /// 设置窗口模式。
    #[must_use]
    pub fn page_mode(mut self, mode: PageMode) -> Self {
        self.page_mode = mode;
        self
    }

    /// 设置页面布局模式。
    #[must_use]
    pub fn page_layout(mut self, layout: PageLayout) -> Self {
        self.page_layout = layout;
        self
    }

    /// 设置标题栏显示模式。
    #[must_use]
    pub fn tab_display(mut self, display: TabDisplay) -> Self {
        self.tab_display = Some(display);
        self
    }

    /// 设置是否隐藏工具栏。
    #[must_use]
    pub fn hide_toolbar(mut self, hide: bool) -> Self {
        self.hide_toolbar = hide;
        self
    }

    /// 设置是否隐藏菜单栏。
    #[must_use]
    pub fn hide_menubar(mut self, hide: bool) -> Self {
        self.hide_menubar = hide;
        self
    }

    /// 设置是否隐藏窗口 UI。
    #[must_use]
    pub fn hide_window_ui(mut self, hide: bool) -> Self {
        self.hide_window_ui = hide;
        self
    }

    /// 设置缩放模式。
    #[must_use]
    pub fn zoom_mode(mut self, mode: ZoomMode) -> Self {
        self.zoom = Some(ZoomScale::Mode(mode));
        self
    }

    /// 设置缩放比例（百分比）。
    #[must_use]
    pub fn zoom_value(mut self, value: f64) -> Self {
        self.zoom = Some(ZoomScale::Value(value));
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;

        let mut xml = String::from("<ofd:VPreferences>");

        let _ = write!(
            xml,
            "\n<ofd:PageMode>{}</ofd:PageMode>",
            self.page_mode.as_str()
        );
        let _ = write!(
            xml,
            "\n<ofd:PageLayout>{}</ofd:PageLayout>",
            self.page_layout.as_str()
        );

        if let Some(ref td) = self.tab_display {
            let _ = write!(xml, "\n<ofd:TabDisplay>{}</ofd:TabDisplay>", td.as_str());
        }

        let _ = write!(
            xml,
            "\n<ofd:HideToolbar>{}</ofd:HideToolbar>",
            self.hide_toolbar
        );
        let _ = write!(
            xml,
            "\n<ofd:HideMenubar>{}</ofd:HideMenubar>",
            self.hide_menubar
        );
        let _ = write!(
            xml,
            "\n<ofd:HideWindowUI>{}</ofd:HideWindowUI>",
            self.hide_window_ui
        );

        match &self.zoom {
            Some(ZoomScale::Mode(m)) => {
                let _ = write!(xml, "\n<ofd:ZoomMode>{}</ofd:ZoomMode>", m.as_str());
            }
            Some(ZoomScale::Value(v)) => {
                let _ = write!(xml, "\n<ofd:Zoom>{v}</ofd:Zoom>");
            }
            None => {}
        }

        xml.push_str("\n</ofd:VPreferences>");
        xml
    }
}

impl Default for CtVPreferences {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_v_preferences_new() {
        let vp = CtVPreferences::new();
        assert_eq!(vp.page_mode, PageMode::None);
        assert_eq!(vp.page_layout, PageLayout::OneColumn);
        assert!(vp.tab_display.is_none());
        assert!(!vp.hide_toolbar);
        assert!(!vp.hide_menubar);
        assert!(!vp.hide_window_ui);
        assert!(vp.zoom.is_none());
    }

    #[test]
    fn test_ct_v_preferences_builder() {
        let vp = CtVPreferences::new()
            .page_mode(PageMode::FullScreen)
            .page_layout(PageLayout::TwoPageLeft)
            .tab_display(TabDisplay::DocTitle)
            .hide_toolbar(true)
            .hide_menubar(true)
            .hide_window_ui(true)
            .zoom_mode(ZoomMode::FitWidth);
        assert_eq!(vp.page_mode, PageMode::FullScreen);
        assert_eq!(vp.page_layout, PageLayout::TwoPageLeft);
        assert_eq!(vp.tab_display, Some(TabDisplay::DocTitle));
        assert!(vp.hide_toolbar);
        assert!(vp.hide_menubar);
        assert!(vp.hide_window_ui);
        assert!(matches!(vp.zoom, Some(ZoomScale::Mode(ZoomMode::FitWidth))));
    }

    #[test]
    fn test_ct_v_preferences_xml() {
        let vp = CtVPreferences::new()
            .page_mode(PageMode::ShowOutline)
            .zoom_value(150.0);
        let xml = vp.to_xml_string();
        assert!(xml.contains("<ofd:PageMode>ShowOutline</ofd:PageMode>"));
        assert!(xml.contains("<ofd:Zoom>150</ofd:Zoom>"));
        assert!(xml.contains("</ofd:VPreferences>"));
    }

    #[test]
    fn test_ct_v_preferences_default() {
        let vp = CtVPreferences::default();
        assert_eq!(vp.page_mode, PageMode::None);
    }

    #[test]
    fn test_page_mode_from_str() {
        assert_eq!(PageMode::from_str("None").unwrap(), PageMode::None);
        assert_eq!(
            PageMode::from_str("FullScreen").unwrap(),
            PageMode::FullScreen
        );
        assert!(PageMode::from_str("Invalid").is_err());
    }

    #[test]
    fn test_page_layout_display() {
        assert_eq!(PageLayout::OneColumn.to_string(), "OneColumn");
        assert_eq!(
            PageLayout::from_str("TwoPageLeft").unwrap(),
            PageLayout::TwoPageLeft
        );
    }
}
