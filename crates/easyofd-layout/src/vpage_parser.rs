//! 虚拟页面 → OFD 页面转换引擎。
//!
//! 将 [`VirtualPage`] 列表转换为 [`OfdPage`] 列表，即将已定位的 Div 盒式模型
//! 映射回 OFD 核心数据模型。嵌套 Children Div 被递归展开为扁平的内容对象列表。
//! 对应 Java: ofdrw-layout `VPageParseEngine`。

use easyofd_core::{ContentObject, ImageObject, OfdPage, TextObject};

use crate::div::{Div, DivContent};
use crate::streaming_layout::VirtualPage;

/// 虚拟页面解析引擎。
///
/// 将 [`VirtualPage`] 列表转为 [`OfdPage`] 列表。
/// 对应 Java: ofdrw-layout `VPageParseEngine`。
#[derive(Debug, Clone, Copy, Default)]
pub struct VPageParseEngine;

impl VPageParseEngine {
    /// 将虚拟页面列表转为 OFD 页面列表。
    ///
    /// 每个 VirtualPage 生成一个 OfdPage，页面尺寸取自 VirtualPage。
    /// Div 递归展开为 ContentObject。
    #[must_use]
    pub fn process(vpages: &[VirtualPage]) -> Vec<OfdPage> {
        vpages
            .iter()
            .map(|vp| {
                let mut page = OfdPage::new(vp.page_width, vp.page_height);
                for div in &vp.divs {
                    collect_content(div, &mut page);
                }
                page
            })
            .collect()
    }
}

/// 递归将 Div 及其子 Div 转为 ContentObject 追加到页面。
fn collect_content(div: &Div, page: &mut OfdPage) {
    match &div.content {
        DivContent::Text(text, style) => {
            let mut t = TextObject::new(div.x, div.y, text.as_str())
                .font(&style.font)
                .size(style.size)
                .color(style.color);
            if style.weight >= 700 {
                t = t.bold();
            }
            if style.italic {
                t = t.italic();
            }
            t.width = Some(div.width);
            t.height = Some(div.height);
            page.add_text(t);
        }
        DivContent::Image { format, .. } => {
            page.add_image(ImageObject::new(
                div.x,
                div.y,
                div.width,
                div.height,
                Vec::new(), // 图片数据由上层填充
                *format,
            ));
        }
        DivContent::Path(path_obj) => {
            let mut p = path_obj.clone();
            p.x = div.x;
            p.y = div.y;
            page.add_path(p);
        }
        DivContent::Children(children) => {
            for child in children {
                collect_content(child, page);
            }
        }
    }
}

/// 辅助函数：将单个 Div 转为 ContentObject 列表（用于测试和外部调用）。
#[must_use]
pub fn div_to_content_objects(div: &Div) -> Vec<ContentObject> {
    let mut page = OfdPage::new(0.0, 0.0);
    collect_content(div, &mut page);
    page.content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::div::TextStyle;
    use easyofd_core::{ImageFormat, PathObject};

    /// 辅助：创建文本 Div。
    fn text_div(text: &str, x: f64, y: f64, w: f64, h: f64) -> Div {
        Div {
            width: w,
            height: h,
            x,
            y,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Text(
                text.to_string(),
                TextStyle {
                    font: "SimSun".to_string(),
                    size: 12.0,
                    weight: 400,
                    italic: false,
                    color: 0,
                },
            ),
        }
    }

    /// 辅助：创建图片 Div。
    fn image_div(x: f64, y: f64, w: f64, h: f64) -> Div {
        Div {
            width: w,
            height: h,
            x,
            y,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Image {
                path: "test.png".to_string(),
                format: ImageFormat::Png,
            },
        }
    }

    // --- 基本文本转换 ---

    #[test]
    fn text_div_becomes_text_object() {
        let div = text_div("hello", 10.0, 20.0, 50.0, 8.0);
        let vpage = VirtualPage {
            divs: vec![div],
            page_width: 210.0,
            page_height: 297.0,
            page_num: None,
        };
        let pages = VPageParseEngine::process(&[vpage]);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].content.len(), 1);
        match &pages[0].content[0] {
            ContentObject::Text(t) => {
                assert_eq!(t.text, "hello");
                assert!((t.x - 10.0).abs() < f64::EPSILON);
                assert!((t.y - 20.0).abs() < f64::EPSILON);
            }
            _ => panic!("期望 TextObject"),
        }
    }

    // --- 图片转换 ---

    #[test]
    fn image_div_becomes_image_object() {
        let div = image_div(5.0, 10.0, 80.0, 60.0);
        let vpage = VirtualPage {
            divs: vec![div],
            page_width: 210.0,
            page_height: 297.0,
            page_num: None,
        };
        let pages = VPageParseEngine::process(&[vpage]);
        match &pages[0].content[0] {
            ContentObject::Image(img) => {
                assert!((img.x - 5.0).abs() < f64::EPSILON);
                assert!((img.width - 80.0).abs() < f64::EPSILON);
                assert_eq!(img.format, ImageFormat::Png);
            }
            _ => panic!("期望 ImageObject"),
        }
    }

    // --- 路径转换 ---

    #[test]
    fn path_div_becomes_path_object() {
        let div = Div {
            width: 0.0,
            height: 0.0,
            x: 3.0,
            y: 4.0,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Path(PathObject::new(0.0, 0.0, "M0 0L10 10")),
        };
        let vpage = VirtualPage {
            divs: vec![div],
            page_width: 210.0,
            page_height: 297.0,
            page_num: None,
        };
        let pages = VPageParseEngine::process(&[vpage]);
        match &pages[0].content[0] {
            ContentObject::Path(p) => {
                assert!((p.x - 3.0).abs() < f64::EPSILON);
                assert!((p.y - 4.0).abs() < f64::EPSILON);
                assert_eq!(p.path_data, "M0 0L10 10");
            }
            _ => panic!("期望 PathObject"),
        }
    }

    // --- 嵌套 Children 展开 ---

    #[test]
    fn children_div_recursively_expanded() {
        let child1 = text_div("a", 0.0, 0.0, 10.0, 5.0);
        let child2 = text_div("b", 0.0, 6.0, 10.0, 5.0);
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
        let vpage = VirtualPage {
            divs: vec![parent],
            page_width: 210.0,
            page_height: 297.0,
            page_num: None,
        };
        let pages = VPageParseEngine::process(&[vpage]);
        // Children 被展开为 2 个 ContentObject
        assert_eq!(pages[0].content.len(), 2);
    }

    // --- 空输入 ---

    #[test]
    fn empty_vpages_returns_empty_pages() {
        let pages = VPageParseEngine::process(&[]);
        assert!(pages.is_empty());
    }

    // --- 多页转换 ---

    #[test]
    fn multiple_vpages_produce_multiple_pages() {
        let vpage1 = VirtualPage {
            divs: vec![text_div("p1", 0.0, 0.0, 50.0, 10.0)],
            page_width: 210.0,
            page_height: 297.0,
            page_num: None,
        };
        let vpage2 = VirtualPage {
            divs: vec![text_div("p2", 0.0, 0.0, 50.0, 10.0)],
            page_width: 297.0,
            page_height: 210.0,
            page_num: None,
        };
        let pages = VPageParseEngine::process(&[vpage1, vpage2]);
        assert_eq!(pages.len(), 2);
        assert!((pages[0].width - 210.0).abs() < f64::EPSILON);
        assert!((pages[1].width - 297.0).abs() < f64::EPSILON);
    }

    // --- 加粗文本 ---

    #[test]
    fn bold_text_div_sets_weight() {
        let div = Div {
            width: 50.0,
            height: 10.0,
            x: 0.0,
            y: 0.0,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: DivContent::Text(
                "标题".to_string(),
                TextStyle {
                    font: "SimHei".to_string(),
                    size: 24.0,
                    weight: 700,
                    italic: false,
                    color: 0xFF_0000,
                },
            ),
        };
        let vpage = VirtualPage {
            divs: vec![div],
            page_width: 210.0,
            page_height: 297.0,
            page_num: None,
        };
        let pages = VPageParseEngine::process(&[vpage]);
        match &pages[0].content[0] {
            ContentObject::Text(t) => {
                assert_eq!(t.weight, 700);
                assert_eq!(t.color, 0xFF_0000);
            }
            _ => panic!("期望 TextObject"),
        }
    }
}
