//! TrueType 字体集合（TTC）解析器。
//!
//! 对应 Java 版 `ofdrw-font` 中的 TTC 集合处理，
//! 解析 TTC 文件头并按索引加载单个 TTF 字体。

use crate::true_type_font::TrueTypeFont;
use crate::ttf_data_stream::TtfDataStream;

/// TTC 文件魔数（`ttcf`）。
const TTC_MAGIC: [u8; 4] = *b"ttcf";

/// TrueType 字体集合。
///
/// TTC（TrueType Collection）文件包含多个 TTF 字体，
/// 通过索引访问单个字体。
#[derive(Debug, Clone)]
pub struct TrueTypeCollection {
    /// TTC 版本号。
    version: u32,
    /// 各字体在 TTC 文件中的偏移量。
    offsets: Vec<u32>,
}

impl TrueTypeCollection {
    /// 从 TTC 原始字节解析字体集合。
    ///
    /// # 参数
    /// - `data`：TTC 文件原始字节
    ///
    /// # 错误
    /// 数据格式无效时返回错误消息。
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let mut stream = TtfDataStream::new(data);

        // 检查魔数
        let tag = stream.read_tag().ok_or("TTC 数据不足，无法读取标签")?;
        if tag != TTC_MAGIC {
            return Err(format!(
                "无效的 TTC 标签: {:?}，期望 {:?}",
                String::from_utf8_lossy(&tag),
                String::from_utf8_lossy(&TTC_MAGIC)
            ));
        }

        let version = stream.read_u32().ok_or("TTC 数据不足，无法读取版本号")?;

        let num_fonts = stream.read_u32().ok_or("TTC 数据不足，无法读取字体数量")? as usize;

        if num_fonts > 1024 {
            return Err(format!("TTC 字体数量异常: {num_fonts}"));
        }

        let mut offsets = Vec::with_capacity(num_fonts);
        for _ in 0..num_fonts {
            let offset = stream.read_u32().ok_or("TTC 数据不足，无法读取字体偏移")?;
            offsets.push(offset);
        }

        Ok(Self { version, offsets })
    }

    /// 返回 TTC 版本号。
    #[must_use]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// 返回集合中的字体数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// 判断集合是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// 获取指定索引字体的偏移量。
    #[must_use]
    pub fn offset(&self, index: usize) -> Option<u32> {
        self.offsets.get(index).copied()
    }

    /// 按索引加载单个 TTF 字体。
    ///
    /// # 参数
    /// - `data`：TTC 文件原始字节
    /// - `index`：字体索引（从 0 开始）
    ///
    /// # 错误
    /// 索引越界或字体解析失败时返回错误。
    pub fn get_font(&self, data: &[u8], index: usize) -> Result<TrueTypeFont, String> {
        let offset = self
            .offsets
            .get(index)
            .ok_or_else(|| format!("字体索引越界: {index}，共 {} 个字体", self.offsets.len()))?;

        let offset = *offset as usize;
        if offset >= data.len() {
            return Err(format!("字体偏移越界: {offset}"));
        }

        TrueTypeFont::parse(&data[offset..])
    }

    /// 返回所有字体偏移量的切片。
    #[must_use]
    pub fn offsets(&self) -> &[u32] {
        &self.offsets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个最小的有效 TTC 文件字节。
    fn build_minimal_ttc(num_fonts: u32) -> Vec<u8> {
        let mut data = Vec::new();

        // TTC 头
        data.extend_from_slice(&TTC_MAGIC);
        data.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // version 1.0
        data.extend_from_slice(&num_fonts.to_be_bytes());

        // 偏移表（每个字体指向文件末尾，即空字体）
        let header_size = 4 + 4 + 4 + (num_fonts * 4);
        for i in 0..num_fonts {
            // 每个字体偏移指向 header_size + i * 最小 TTF 大小
            let font_offset = header_size + i * 12; // 12 = 最小偏移表大小
            data.extend_from_slice(&font_offset.to_be_bytes());
        }

        // 写入 num_fonts 个最小 TTF（仅偏移表）
        for _ in 0..num_fonts {
            data.extend_from_slice(&crate::true_type_font::TTF_MAGIC.to_be_bytes());
            data.extend_from_slice(&0u16.to_be_bytes()); // numTables = 0
            data.extend_from_slice(&0u16.to_be_bytes()); // searchRange
            data.extend_from_slice(&0u16.to_be_bytes()); // entrySelector
            data.extend_from_slice(&0u16.to_be_bytes()); // rangeShift
        }

        data
    }

    #[test]
    fn test_parse_single_font_ttc() {
        let data = build_minimal_ttc(1);
        let ttc = TrueTypeCollection::parse(&data).unwrap();
        assert_eq!(ttc.len(), 1);
        assert_eq!(ttc.version(), 0x0001_0000);
        assert!(!ttc.is_empty());
    }

    #[test]
    fn test_parse_multi_font_ttc() {
        let data = build_minimal_ttc(3);
        let ttc = TrueTypeCollection::parse(&data).unwrap();
        assert_eq!(ttc.len(), 3);
    }

    #[test]
    fn test_get_font() {
        let data = build_minimal_ttc(2);
        let ttc = TrueTypeCollection::parse(&data).unwrap();
        let font = ttc.get_font(&data, 0).unwrap();
        assert_eq!(font.num_tables(), 0);

        let font2 = ttc.get_font(&data, 1).unwrap();
        assert_eq!(font2.num_tables(), 0);
    }

    #[test]
    fn test_get_font_index_out_of_bounds() {
        let data = build_minimal_ttc(1);
        let ttc = TrueTypeCollection::parse(&data).unwrap();
        let result = ttc.get_font(&data, 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("越界"));
    }

    #[test]
    fn test_parse_invalid_tag() {
        let mut data = Vec::new();
        data.extend_from_slice(b"NOTA");
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&0u32.to_be_bytes());
        let result = TrueTypeCollection::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效"));
    }

    #[test]
    fn test_offset() {
        let data = build_minimal_ttc(2);
        let ttc = TrueTypeCollection::parse(&data).unwrap();
        assert!(ttc.offset(0).is_some());
        assert!(ttc.offset(1).is_some());
        assert!(ttc.offset(5).is_none());
    }

    #[test]
    fn test_offsets_slice() {
        let data = build_minimal_ttc(3);
        let ttc = TrueTypeCollection::parse(&data).unwrap();
        assert_eq!(ttc.offsets().len(), 3);
    }
}
