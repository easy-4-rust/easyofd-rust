/// 从固定版式页面推断出的语义块。
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutBlock {
    /// 标题块。
    Heading {
        /// 标题级别，范围 1 到 6。
        level: u8,
        /// 标题文本。
        text: String,
        /// 来源对象下标。
        source_indices: Vec<usize>,
    },
    /// 普通段落或文本行。
    Paragraph {
        /// 段落文本。
        text: String,
        /// 来源对象下标。
        source_indices: Vec<usize>,
    },
    /// 图片对象。
    Image {
        /// 来源对象下标。
        source_index: usize,
    },
}
