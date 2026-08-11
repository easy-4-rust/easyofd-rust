//! HTML 元素（简化值类型）。
//!
//! 对应 Java: org.ofdrw.converter.html.Element
//!
//! Java 版 `Element` 是一个完整的 HTML DOM 节点（1383 处引用），
//! 依赖 Java AWT 渲染管线。Rust 版移植为**简化值类型**，
//! 仅保留转换过程中需要的标签名、属性和文本内容，
//! 不包含渲染/布局逻辑。

use std::collections::HashMap;

/// HTML 元素。
///
/// 对应 Java `Element`（简化版）。用于 OFD → HTML 转换时
/// 构建 HTML DOM 树的中间表示。
///
/// 与 Java 版的主要差异：
/// - 不继承任何基类，使用组合而非继承
/// - 子节点使用 `Vec<Element>` 而非 Java 的 NodeList
/// - 不包含渲染/绘制方法
#[derive(Debug, Clone)]
pub struct Element {
    /// 标签名（如 "div"、"span"、"p"）。
    tag_name: String,
    /// 属性映射。
    attributes: HashMap<String, String>,
    /// 文本内容（叶子节点）。
    text_content: Option<String>,
    /// 子元素。
    children: Vec<Element>,
    /// CSS 类名列表。
    class_names: Vec<String>,
    /// 内联样式。
    style: Option<String>,
}

impl Element {
    /// 创建元素。
    ///
    /// # 参数
    /// - `tag_name`：标签名
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes: HashMap::new(),
            text_content: None,
            children: Vec::new(),
            class_names: Vec::new(),
            style: None,
        }
    }

    /// 创建文本节点。
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            tag_name: String::new(),
            attributes: HashMap::new(),
            text_content: Some(content.into()),
            children: Vec::new(),
            class_names: Vec::new(),
            style: None,
        }
    }

    // ─── 标签名 ──────────────────────────────────────────────────────────────

    /// 返回标签名。
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    // ─── 属性 ────────────────────────────────────────────────────────────────

    /// 设置属性。
    pub fn set_attr(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// 获取属性值。
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    /// 是否包含属性。
    pub fn has_attr(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// 返回所有属性。
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    // ─── 文本内容 ────────────────────────────────────────────────────────────

    /// 返回文本内容。
    pub fn text_content(&self) -> Option<&str> {
        self.text_content.as_deref()
    }

    /// 设置文本内容。
    pub fn set_text_content(&mut self, content: impl Into<String>) {
        self.text_content = Some(content.into());
    }

    /// 是否为文本节点。
    pub fn is_text_node(&self) -> bool {
        self.tag_name.is_empty() && self.text_content.is_some()
    }

    // ─── 子元素 ──────────────────────────────────────────────────────────────

    /// 追加子元素。
    pub fn append_child(&mut self, child: Element) {
        self.children.push(child);
    }

    /// 返回子元素列表。
    pub fn children(&self) -> &[Element] {
        &self.children
    }

    /// 返回可变子元素列表。
    pub fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }

    /// 子元素数量。
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    // ─── CSS 类名 ────────────────────────────────────────────────────────────

    /// 添加 CSS 类名。
    pub fn add_class(&mut self, class: impl Into<String>) {
        self.class_names.push(class.into());
    }

    /// 返回 CSS 类名列表。
    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }

    /// 是否包含指定 CSS 类名。
    pub fn has_class(&self, class: &str) -> bool {
        self.class_names.iter().any(|c| c == class)
    }

    // ─── 内联样式 ────────────────────────────────────────────────────────────

    /// 设置内联样式。
    pub fn set_style(&mut self, style: impl Into<String>) {
        self.style = Some(style.into());
    }

    /// 返回内联样式。
    pub fn style(&self) -> Option<&str> {
        self.style.as_deref()
    }

    // ─── 序列化 ──────────────────────────────────────────────────────────────

    /// 序列化为 HTML 字符串（简易实现）。
    pub fn to_html(&self) -> String {
        if self.is_text_node() {
            return self.text_content.clone().unwrap_or_default();
        }

        let mut html = format!("<{}", self.tag_name);

        // 属性
        for (key, value) in &self.attributes {
            html.push_str(&format!(" {key}=\"{value}\""));
        }

        // CSS 类名
        if !self.class_names.is_empty() {
            html.push_str(&format!(" class=\"{}\"", self.class_names.join(" ")));
        }

        // 内联样式
        if let Some(style) = &self.style {
            html.push_str(&format!(" style=\"{style}\""));
        }

        html.push('>');

        // 子元素 / 文本
        if let Some(text) = &self.text_content {
            html.push_str(text);
        }
        for child in &self.children {
            html.push_str(&child.to_html());
        }

        html.push_str(&format!("</{}>", self.tag_name));
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_element() {
        let el = Element::new("div");
        assert_eq!(el.tag_name(), "div");
        assert!(el.text_content().is_none());
        assert_eq!(el.child_count(), 0);
    }

    #[test]
    fn test_text_node() {
        let el = Element::text("hello");
        assert!(el.is_text_node());
        assert_eq!(el.text_content(), Some("hello"));
        assert_eq!(el.to_html(), "hello");
    }

    #[test]
    fn test_attributes() {
        let mut el = Element::new("input");
        el.set_attr("type", "text");
        el.set_attr("value", "test");
        assert_eq!(el.attr("type"), Some("text"));
        assert!(el.has_attr("value"));
        assert!(!el.has_attr("name"));
    }

    #[test]
    fn test_children() {
        let mut parent = Element::new("div");
        parent.append_child(Element::text("child1"));
        parent.append_child(Element::new("span"));
        assert_eq!(parent.child_count(), 2);
        assert_eq!(parent.children()[0].text_content(), Some("child1"));
    }

    #[test]
    fn test_css_class() {
        let mut el = Element::new("p");
        el.add_class("highlight");
        el.add_class("bold");
        assert!(el.has_class("highlight"));
        assert!(el.has_class("bold"));
        assert!(!el.has_class("italic"));
        assert_eq!(el.class_names().len(), 2);
    }

    #[test]
    fn test_style() {
        let mut el = Element::new("div");
        assert!(el.style().is_none());
        el.set_style("color: red");
        assert_eq!(el.style(), Some("color: red"));
    }

    #[test]
    fn test_to_html_simple() {
        let el = Element::new("br");
        assert_eq!(el.to_html(), "<br></br>");
    }

    #[test]
    fn test_to_html_with_children() {
        let mut div = Element::new("div");
        div.set_attr("id", "main");
        div.append_child(Element::text("hello"));
        let mut span = Element::new("span");
        span.append_child(Element::text("world"));
        div.append_child(span);
        let html = div.to_html();
        assert!(html.contains("<div id=\"main\">"));
        assert!(html.contains("hello"));
        assert!(html.contains("<span>world</span>"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn test_to_html_with_class_and_style() {
        let mut el = Element::new("p");
        el.add_class("intro");
        el.set_style("font-size: 16px");
        el.set_text_content("Hello");
        let html = el.to_html();
        assert!(html.contains("class=\"intro\""));
        assert!(html.contains("style=\"font-size: 16px\""));
    }
}
