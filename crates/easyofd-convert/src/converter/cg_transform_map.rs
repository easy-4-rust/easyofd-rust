//! 字符到字形变换映射。
//!
//! 对应 Java: org.ofdrw.converter.CGTransformMap

/// 字符到字形变换映射。
///
/// 对应 Java `CGTransformMap`。用于记录 OFD 文本对象中
/// 字符编码到字形 ID 的映射关系（CGTransform）。
///
/// 在 OFD 规范中，`TextCode` 的 `CGTransform` 元素定义了
/// 从字符序列到字形序列的映射规则。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CgTransformEntry {
    /// 字符位置（在 TextCode 中的起始索引）。
    pub code_position: u16,
    /// 字符数量。
    pub code_count: u16,
    /// 字形数量。
    pub glyph_count: u16,
    /// 字形 ID 列表。
    pub glyph_ids: Vec<String>,
}

impl CgTransformEntry {
    /// 创建变换条目。
    pub fn new(
        code_position: u16,
        code_count: u16,
        glyph_count: u16,
        glyph_ids: Vec<String>,
    ) -> Self {
        Self {
            code_position,
            code_count,
            glyph_count,
            glyph_ids,
        }
    }
}

/// 字符到字形变换映射集合。
///
/// 对应 Java `CGTransformMap`。管理一组 `CgTransformEntry`，
/// 提供按字符位置查找的能力。
#[derive(Debug, Clone, Default)]
pub struct CgTransformMap {
    /// 变换条目列表。
    entries: Vec<CgTransformEntry>,
}

impl CgTransformMap {
    /// 创建空的变换映射。
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 从条目列表创建。
    pub fn from_entries(entries: Vec<CgTransformEntry>) -> Self {
        Self { entries }
    }

    /// 添加变换条目。
    pub fn push(&mut self, entry: CgTransformEntry) {
        self.entries.push(entry);
    }

    /// 返回所有条目。
    pub fn entries(&self) -> &[CgTransformEntry] {
        &self.entries
    }

    /// 条目数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 查找包含指定字符位置的变换条目。
    pub fn find_by_code_position(&self, position: u16) -> Option<&CgTransformEntry> {
        self.entries
            .iter()
            .find(|e| position >= e.code_position && position < e.code_position + e.code_count)
    }

    /// 判断指定字符位置是否需要跳过（即在 CGTransform 范围内但不是第一个字符）。
    pub fn should_skip_position(&self, position: u16) -> bool {
        if let Some(entry) = self.find_by_code_position(position) {
            // 如果字形数量少于字符数量，中间的字符位置需要跳过
            entry.glyph_count < entry.code_count && position > entry.code_position
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let map = CgTransformMap::new();
        assert!(map.is_empty());
    }

    #[test]
    fn test_push_and_len() {
        let mut map = CgTransformMap::new();
        map.push(CgTransformEntry::new(0, 2, 1, vec!["glyph1".to_string()]));
        map.push(CgTransformEntry::new(
            3,
            3,
            3,
            vec!["g2".to_string(), "g3".to_string(), "g4".to_string()],
        ));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_find_by_code_position() {
        let mut map = CgTransformMap::new();
        map.push(CgTransformEntry::new(
            5,
            3,
            3,
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        ));

        assert!(map.find_by_code_position(4).is_none());
        assert!(map.find_by_code_position(5).is_some());
        assert!(map.find_by_code_position(7).is_some());
        assert!(map.find_by_code_position(8).is_none());
    }

    #[test]
    fn test_should_skip_position() {
        let mut map = CgTransformMap::new();
        // 2 个字符映射到 1 个字形
        map.push(CgTransformEntry::new(0, 2, 1, vec!["glyph1".to_string()]));

        assert!(!map.should_skip_position(0)); // 第一个字符不跳过
        assert!(map.should_skip_position(1)); // 第二个字符跳过
        assert!(!map.should_skip_position(2)); // 范围外不跳过
    }

    #[test]
    fn test_should_skip_position_equal_counts() {
        let mut map = CgTransformMap::new();
        // 3 个字符映射到 3 个字形（一一对应，不跳过）
        map.push(CgTransformEntry::new(
            0,
            3,
            3,
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        ));

        assert!(!map.should_skip_position(0));
        assert!(!map.should_skip_position(1));
        assert!(!map.should_skip_position(2));
    }

    #[test]
    fn test_from_entries() {
        let entries = vec![CgTransformEntry::new(0, 1, 1, vec!["x".to_string()])];
        let map = CgTransformMap::from_entries(entries);
        assert_eq!(map.len(), 1);
    }
}
