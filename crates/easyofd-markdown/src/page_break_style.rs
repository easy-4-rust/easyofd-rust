/// Markdown 页面分隔策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageBreakStyle {
    /// 使用 HTML 注释保留页码，不影响渲染。
    #[default]
    HtmlComment,
    /// 使用 Markdown 水平线。
    HorizontalRule,
    /// 不生成页面分隔。
    None,
}
