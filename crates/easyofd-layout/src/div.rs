//! Div 盒式模型（GB/T 33190 第 9.1 节 + CSS 盒式模型 + ofdrw-layout.element.Div）。
//!
//! 提供固定版式内容对象到可布局盒式模型的映射，支持文本、图片、矢量路径
//! 以及嵌套子 Div。盒式模型的 `x` / `y` 坐标在布局引擎计算后填充。

use easyofd_core::{ContentObject, ImageFormat, ImageObject, PathObject, TextObject};

/// 文本样式，从 [`TextObject`] 提取，便于在盒式模型中独立传递。
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// 字体族名。
    pub font: String,
    /// 字号（pt）。
    pub size: f64,
    /// 字重：400 = 正常，700 = 加粗。
    pub weight: u32,
    /// 是否斜体。
    pub italic: bool,
    /// 文本颜色（RGB）。
    pub color: u32,
}

impl TextStyle {
    /// 从 [`TextObject`] 提取样式信息。
    #[must_use]
    pub fn from_text_object(t: &TextObject) -> Self {
        Self {
            font: t.font.clone(),
            size: t.size,
            weight: t.weight,
            italic: t.italic,
            color: t.color,
        }
    }
}

/// Div 盒式模型。
///
/// 参照 CSS 盒式模型：外边距（margin）+ 边框（border）+ 内边距（padding）+ 内容。
/// 尺寸单位均为毫米（mm），与 OFD 页面坐标系一致。
#[derive(Debug, Clone)]
pub struct Div {
    /// 内容区宽度（mm）。
    pub width: f64,
    /// 内容区高度（mm）。
    pub height: f64,
    /// 左上角 X 坐标（mm），由布局引擎计算后填充。
    pub x: f64,
    /// 左上角 Y 坐标（mm），由布局引擎计算后填充。
    pub y: f64,
    /// 四周内边距（mm）。
    pub padding: f64,
    /// 四周边框宽度（mm）。
    pub border: f64,
    /// 四周外边距（mm）。
    pub margin: f64,
    /// 背景色（RGB），`None` 表示透明。
    pub background: Option<u32>,
    /// 盒式模型内容。
    pub content: DivContent,
}

/// Div 可容纳的内容类型。
#[derive(Debug, Clone)]
pub enum DivContent {
    /// 文本内容及样式。
    Text(String, TextStyle),
    /// 图片内容。
    Image {
        /// 图片路径标识。
        path: String,
        /// 图片格式。
        format: ImageFormat,
    },
    /// 矢量路径对象（直接引用核心类型）。
    Path(PathObject),
    /// 嵌套子 Div 列表。
    Children(Vec<Div>),
}

impl Div {
    /// 从 [`ContentObject`] 创建 Div。
    ///
    /// 使用对象自身的几何尺寸作为盒式模型的宽高，位置由 `x`/`y` 决定。
    #[must_use]
    pub fn from_content_object(obj: &ContentObject) -> Self {
        match obj {
            ContentObject::Text(t) => Self::from_text_object(t),
            ContentObject::Image(i) => Self::from_image_object(i),
            ContentObject::Path(p) => Self::from_path_object(p),
        }
    }

    /// 从 [`TextObject`] 创建文本 Div。
    #[must_use]
    pub fn from_text_object(t: &TextObject) -> Self {
        let width = t
            .width
            .unwrap_or_else(|| estimate_text_width(t.text.chars().count(), t.size));
        let height = t.height.unwrap_or(t.size * 0.352_8); // pt -> mm
        Self {
            width,
            height,
            x: t.x,
            y: t.y,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Text(t.text.clone(), TextStyle::from_text_object(t)),
        }
    }

    /// 从 [`ImageObject`] 创建图片 Div。
    #[must_use]
    pub fn from_image_object(i: &ImageObject) -> Self {
        Self {
            width: i.width,
            height: i.height,
            x: i.x,
            y: i.y,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Image {
                path: String::new(),
                format: i.format,
            },
        }
    }

    /// 从 [`PathObject`] 创建路径 Div。
    #[must_use]
    pub fn from_path_object(p: &PathObject) -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            x: p.x,
            y: p.y,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Path(p.clone()),
        }
    }

    /// 计算包含内边距和边框的总宽度（不含外边距）。
    #[must_use]
    pub fn outer_width(&self) -> f64 {
        self.width + 2.0 * (self.padding + self.border)
    }

    /// 计算包含内边距和边框的总高度（不含外边距）。
    #[must_use]
    pub fn outer_height(&self) -> f64 {
        self.height + 2.0 * (self.padding + self.border)
    }

    /// 计算含外边距的总宽度。
    #[must_use]
    pub fn margin_box_width(&self) -> f64 {
        self.outer_width() + 2.0 * self.margin
    }

    /// 计算含外边距的总高度。
    #[must_use]
    pub fn margin_box_height(&self) -> f64 {
        self.outer_height() + 2.0 * self.margin
    }

    /// 提取文本内容（如果是文本 Div）。
    #[must_use]
    pub fn text_content(&self) -> Option<&str> {
        match &self.content {
            DivContent::Text(text, _) => Some(text),
            _ => None,
        }
    }

    /// 递归收集所有文本内容，拼接为单个字符串（深度优先）。
    #[must_use]
    pub fn collect_text(&self) -> String {
        let mut parts = Vec::new();
        collect_text_recursive(self, &mut parts);
        parts.join("")
    }

    /// 递归统计 Div 树中的节点总数。
    #[must_use]
    pub fn count_nodes(&self) -> usize {
        match &self.content {
            DivContent::Children(children) => {
                1 + children.iter().map(Self::count_nodes).sum::<usize>()
            }
            _ => 1,
        }
    }
}

/// 估算文本宽度（mm），基于字符数和字号。
fn estimate_text_width(char_count: usize, size_pt: f64) -> f64 {
    let chars = u32::try_from(char_count).unwrap_or(u32::MAX);
    f64::from(chars) * size_pt * 0.06
}

/// 递归收集文本。
fn collect_text_recursive(div: &Div, out: &mut Vec<String>) {
    match &div.content {
        DivContent::Text(text, _) => {
            out.push(text.clone());
        }
        DivContent::Children(children) => {
            for child in children {
                collect_text_recursive(child, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 文本 Div 构造 ---

    #[test]
    fn div_from_text_uses_object_dimensions() {
        let t = TextObject::new(10.0, 20.0, "hello")
            .size(24.0)
            .bold()
            .color(0xFF_0000);
        let div = Div::from_text_object(&t);
        assert!((div.x - 10.0).abs() < f64::EPSILON);
        assert!((div.y - 20.0).abs() < f64::EPSILON);
        assert!(div.width > 0.0);
        assert!(div.height > 0.0);
        let text = div.text_content().expect("应为文本 Div");
        assert_eq!(text, "hello");
    }

    // --- 图片 Div 构造 ---

    #[test]
    fn div_from_image_preserves_dimensions() {
        let img = ImageObject::png(5.0, 10.0, 80.0, 60.0, vec![0x89]);
        let div = Div::from_image_object(&img);
        assert!((div.width - 80.0).abs() < f64::EPSILON);
        assert!((div.height - 60.0).abs() < f64::EPSILON);
        assert!(matches!(
            div.content,
            DivContent::Image {
                format: ImageFormat::Png,
                ..
            }
        ));
    }

    // --- 路径 Div 构造 ---

    #[test]
    fn div_from_path_copies_position() {
        let p = PathObject::new(3.0, 4.0, "M0 0L10 10");
        let div = Div::from_path_object(&p);
        assert!((div.x - 3.0).abs() < f64::EPSILON);
        assert!((div.y - 4.0).abs() < f64::EPSILON);
        assert!(matches!(div.content, DivContent::Path(_)));
    }

    // --- ContentObject 分发 ---

    #[test]
    fn div_from_content_object_dispatches_text() {
        let obj = ContentObject::Text(TextObject::new(0.0, 0.0, "abc"));
        let div = Div::from_content_object(&obj);
        assert!(div.text_content().is_some());
    }

    #[test]
    fn div_from_content_object_dispatches_image() {
        let obj = ContentObject::Image(ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]));
        let div = Div::from_content_object(&obj);
        assert!(div.text_content().is_none());
    }

    #[test]
    fn div_from_content_object_dispatches_path() {
        let obj = ContentObject::Path(PathObject::hline(0.0, 0.0, 100.0));
        let div = Div::from_content_object(&obj);
        assert!(matches!(div.content, DivContent::Path(_)));
    }

    // --- 盒式模型尺寸计算 ---

    #[test]
    fn div_outer_dimensions_include_padding_and_border() {
        let t = TextObject::new(0.0, 0.0, "x");
        let mut div = Div::from_text_object(&t);
        div.padding = 2.0;
        div.border = 1.0;
        let expected_w = div.width + 2.0 * (2.0 + 1.0);
        let expected_h = div.height + 2.0 * (2.0 + 1.0);
        assert!((div.outer_width() - expected_w).abs() < f64::EPSILON);
        assert!((div.outer_height() - expected_h).abs() < f64::EPSILON);
    }

    #[test]
    fn div_margin_box_adds_margin() {
        let t = TextObject::new(0.0, 0.0, "x");
        let mut div = Div::from_text_object(&t);
        div.padding = 1.0;
        div.border = 0.5;
        div.margin = 3.0;
        let expected = div.outer_width() + 2.0 * 3.0;
        assert!((div.margin_box_width() - expected).abs() < f64::EPSILON);
    }

    // --- 嵌套子 Div ---

    #[test]
    fn div_children_nested() {
        let child1 = Div::from_text_object(&TextObject::new(0.0, 0.0, "a"));
        let child2 = Div::from_text_object(&TextObject::new(0.0, 10.0, "b"));
        let parent = Div {
            width: 100.0,
            height: 50.0,
            x: 0.0,
            y: 0.0,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Children(vec![child1, child2]),
        };
        assert_eq!(parent.count_nodes(), 3); // parent + 2 children
        assert_eq!(parent.collect_text(), "ab");
    }

    // --- collect_text 递归 ---

    #[test]
    fn div_collect_text_mixed_content() {
        let text_div = Div::from_text_object(&TextObject::new(0.0, 0.0, "hello "));
        let img_div = Div::from_image_object(&ImageObject::jpeg(0.0, 0.0, 1.0, 1.0, vec![0]));
        let nested_text = Div::from_text_object(&TextObject::new(0.0, 0.0, "world"));
        let parent = Div {
            width: 100.0,
            height: 50.0,
            x: 0.0,
            y: 0.0,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Children(vec![text_div, img_div, nested_text]),
        };
        assert_eq!(parent.collect_text(), "hello world");
        // 图片不贡献文本
    }

    // --- background 设置 ---

    #[test]
    fn div_with_background() {
        let mut div = Div::from_text_object(&TextObject::new(0.0, 0.0, "x"));
        div.background = Some(0xFF_FF00);
        assert_eq!(div.background, Some(0xFF_FF00));
    }

    // --- count_nodes 叶子 ---

    #[test]
    fn div_count_nodes_leaf() {
        let div = Div::from_text_object(&TextObject::new(0.0, 0.0, "leaf"));
        assert_eq!(div.count_nodes(), 1);
    }
}
