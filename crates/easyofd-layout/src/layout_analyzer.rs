use std::cmp::Ordering;

use easyofd_core::{ContentObject, OfdPage, TextObject};

use crate::{LayoutBlock, LayoutOptions, LayoutResult};

/// 基于对象几何坐标恢复确定性阅读顺序的分析器。
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutAnalyzer {
    options: LayoutOptions,
}

impl LayoutAnalyzer {
    /// 使用指定选项创建分析器。
    #[must_use]
    pub fn new(options: LayoutOptions) -> Self {
        Self { options }
    }

    /// 分析单页并返回语义块。
    #[must_use]
    pub fn analyze_page(&self, page_number: usize, page: &OfdPage) -> LayoutResult {
        let mut text_objects: Vec<(usize, &TextObject)> = page
            .content
            .iter()
            .enumerate()
            .filter_map(|(index, object)| match object {
                ContentObject::Text(text) => Some((index, text)),
                _ => None,
            })
            .collect();
        text_objects.sort_by(|left, right| {
            left.1
                .y
                .partial_cmp(&right.1.y)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.1.x.partial_cmp(&right.1.x).unwrap_or(Ordering::Equal))
        });

        let mut lines: Vec<Vec<(usize, &TextObject)>> = Vec::new();
        for item in text_objects {
            if let Some(line) = lines.last_mut()
                && line.first().is_some_and(|(_, first)| {
                    (first.y - item.1.y).abs() <= self.options.line_tolerance
                })
            {
                line.push(item);
            } else {
                lines.push(vec![item]);
            }
        }

        let mut positioned = Vec::new();
        for mut line in lines {
            line.sort_by(|left, right| left.1.x.partial_cmp(&right.1.x).unwrap_or(Ordering::Equal));
            let y = line.first().map_or(0.0, |(_, text)| text.y);
            let mut value = String::new();
            let mut previous_end = None;
            let mut maximum_size = 0.0_f64;
            let mut bold = false;
            let mut indices = Vec::with_capacity(line.len());
            for (index, text) in line {
                if previous_end.is_some_and(|end| text.x - end >= self.options.word_gap)
                    && !value.chars().last().is_some_and(char::is_whitespace)
                {
                    value.push(' ');
                }
                value.push_str(text.text.trim());
                previous_end = Some(text.x + estimated_width(text));
                maximum_size = maximum_size.max(text.size);
                bold |= text.weight >= 700;
                indices.push(index);
            }
            let block =
                if maximum_size >= self.options.heading_size || (bold && maximum_size >= 14.0) {
                    LayoutBlock::Heading {
                        level: heading_level(maximum_size, self.options.heading_size),
                        text: value,
                        source_indices: indices,
                    }
                } else {
                    LayoutBlock::Paragraph {
                        text: value,
                        source_indices: indices,
                    }
                };
            positioned.push((y, block));
        }

        for (index, object) in page.content.iter().enumerate() {
            if let ContentObject::Image(image) = object {
                positioned.push((
                    image.y,
                    LayoutBlock::Image {
                        source_index: index,
                    },
                ));
            }
        }
        positioned.sort_by(|left, right| left.0.partial_cmp(&right.0).unwrap_or(Ordering::Equal));

        let path_count = page
            .content
            .iter()
            .filter(|object| matches!(object, ContentObject::Path(_)))
            .count();
        let warnings = if path_count == 0 {
            Vec::new()
        } else {
            vec![format!(
                "page {page_number}: {path_count} vector path object(s) are not represented as Markdown"
            )]
        };
        LayoutResult {
            page_number,
            blocks: positioned.into_iter().map(|(_, block)| block).collect(),
            warnings,
        }
    }
}

fn estimated_width(text: &TextObject) -> f64 {
    text.width.unwrap_or_else(|| {
        let characters = f64::from(u32::try_from(text.text.chars().count()).unwrap_or(u32::MAX));
        characters * text.size * 0.06
    })
}

fn heading_level(size: f64, base: f64) -> u8 {
    if size >= base * 1.7 {
        1
    } else if size >= base * 1.35 {
        2
    } else if size >= base {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::TextObject;

    #[test]
    fn restores_geometric_order_and_heading() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(50.0, 30.0, "world"));
        page.add_text(TextObject::new(10.0, 10.0, "Title").size(24.0));
        page.add_text(TextObject::new(10.0, 30.0, "hello"));
        let result = LayoutAnalyzer::default().analyze_page(1, &page);
        assert!(matches!(result.blocks[0], LayoutBlock::Heading { .. }));
        assert!(matches!(
            &result.blocks[1],
            LayoutBlock::Paragraph { text, .. } if text == "hello world"
        ));
    }
}
