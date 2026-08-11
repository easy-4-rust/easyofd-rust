//! 字形数据提供器 trait。
//!
//! 对应 Java: org.ofdrw.converter.font.GlyphDataProvider

/// 字形数据提供器。
///
/// 对应 Java `GlyphDataProvider` 接口。用于在解析复合字形（composite glyph）
/// 时获取子字形的数据。
pub trait GlyphDataProvider {
    /// 读取指定索引的字形轮廓数量。
    ///
    /// # 参数
    /// - `glyph_index`：字形索引
    fn get_contour_count(&self, glyph_index: u16) -> i16;

    /// 读取指定索引的字形数据。
    ///
    /// # 参数
    /// - `glyph_index`：字形索引
    ///
    /// # 返回
    /// 字形数据的原始字节，如果不存在则返回 `None`。
    fn get_glyph_data(&self, glyph_index: u16) -> Option<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider;

    impl GlyphDataProvider for MockProvider {
        fn get_contour_count(&self, _glyph_index: u16) -> i16 {
            2
        }

        fn get_glyph_data(&self, glyph_index: u16) -> Option<Vec<u8>> {
            if glyph_index < 10 {
                Some(vec![0x00, 0x01])
            } else {
                None
            }
        }
    }

    #[test]
    fn test_mock_provider() {
        let provider = MockProvider;
        assert_eq!(provider.get_contour_count(0), 2);
        assert!(provider.get_glyph_data(5).is_some());
        assert!(provider.get_glyph_data(10).is_none());
    }
}
