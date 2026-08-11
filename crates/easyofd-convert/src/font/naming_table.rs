//! 字体命名表。
//!
//! 对应 Java: org.ofdrw.converter.font.NamingTable
//!
//! 参考 Apache FontBox，遵循 OpenType `name` 表规范。

use crate::font::name_record::{
    ENCODING_MACINTOSH_ROMAN, ENCODING_WINDOWS_UNICODE_BMP, LANGUAGE_MACINTOSH_ENGLISH,
    LANGUAGE_UNICODE, LANGUAGE_WINDOWS_EN_US, NAME_FONT_FAMILY_NAME, NAME_FONT_SUB_FAMILY_NAME,
    NAME_POSTSCRIPT_NAME, NameRecord, PLATFORM_MACINTOSH, PLATFORM_UNICODE, PLATFORM_WINDOWS,
};
use std::collections::HashMap;

/// 四维查找表：name_id → platform_id → encoding_id → language_id → string。
type LookupTable = HashMap<u16, HashMap<u16, HashMap<u16, HashMap<u16, String>>>>;

/// 字体命名表。
///
/// 对应 Java `NamingTable`。解析 OpenType `name` 表，提供按名称 ID、
/// 平台、编码、语言查找字体名称的能力。
#[derive(Debug, Clone)]
pub struct NamingTable {
    /// 名称记录列表。
    name_records: Vec<NameRecord>,
    /// 四维查找表。
    lookup_table: LookupTable,
    /// 字体族名（英文）。
    font_family: Option<String>,
    /// 字体子族名（英文）。
    font_sub_family: Option<String>,
    /// PostScript 名称。
    post_script_name: Option<String>,
}

impl NamingTable {
    /// 创建空的命名表。
    pub fn new() -> Self {
        Self {
            name_records: Vec::new(),
            lookup_table: HashMap::new(),
            font_family: None,
            font_sub_family: None,
            post_script_name: None,
        }
    }

    /// 从名称记录列表构建命名表。
    ///
    /// 自动构建查找表并提取常用名称。
    pub fn from_records(records: Vec<NameRecord>) -> Self {
        let mut table = Self {
            name_records: records,
            lookup_table: HashMap::new(),
            font_family: None,
            font_sub_family: None,
            post_script_name: None,
        };
        table.build_lookup_table();
        table.extract_common_names();
        table
    }

    /// 构建四维查找表。
    fn build_lookup_table(&mut self) {
        self.lookup_table.clear();
        for nr in &self.name_records {
            let string = match nr.string_value() {
                Some(s) => s.to_string(),
                None => continue,
            };
            self.lookup_table
                .entry(nr.name_id())
                .or_default()
                .entry(nr.platform_id())
                .or_default()
                .entry(nr.platform_encoding_id())
                .or_default()
                .insert(nr.language_id(), string);
        }
    }

    /// 提取常用名称（font_family、font_sub_family、post_script_name）。
    fn extract_common_names(&mut self) {
        self.font_family = self.get_english_name(NAME_FONT_FAMILY_NAME);
        self.font_sub_family = self.get_english_name(NAME_FONT_SUB_FAMILY_NAME);

        // PostScript 名称：优先 Macintosh，其次 Windows
        self.post_script_name = self
            .get_name(
                NAME_POSTSCRIPT_NAME,
                PLATFORM_MACINTOSH,
                ENCODING_MACINTOSH_ROMAN,
                LANGUAGE_MACINTOSH_ENGLISH,
            )
            .or_else(|| {
                self.get_name(
                    NAME_POSTSCRIPT_NAME,
                    PLATFORM_WINDOWS,
                    ENCODING_WINDOWS_UNICODE_BMP,
                    LANGUAGE_WINDOWS_EN_US,
                )
            })
            .map(|s| s.trim().to_string());
    }

    /// 获取英文名称（优先级：Unicode 2.0 Full > BMP > 1.1 > 1.0 > Windows > Macintosh）。
    fn get_english_name(&self, name_id: u16) -> Option<String> {
        // Unicode 编码 ID: 4, 3, 1, 0
        for encoding_id in [4u16, 3, 1, 0] {
            if let Some(name) =
                self.get_name(name_id, PLATFORM_UNICODE, encoding_id, LANGUAGE_UNICODE)
            {
                return Some(name);
            }
        }
        // Windows
        if let Some(name) = self.get_name(
            name_id,
            PLATFORM_WINDOWS,
            ENCODING_WINDOWS_UNICODE_BMP,
            LANGUAGE_WINDOWS_EN_US,
        ) {
            return Some(name);
        }
        // Macintosh
        self.get_name(
            name_id,
            PLATFORM_MACINTOSH,
            ENCODING_MACINTOSH_ROMAN,
            LANGUAGE_MACINTOSH_ENGLISH,
        )
    }

    /// 从查找表中获取名称。
    pub fn get_name(
        &self,
        name_id: u16,
        platform_id: u16,
        encoding_id: u16,
        language_id: u16,
    ) -> Option<String> {
        self.lookup_table
            .get(&name_id)
            .and_then(|p| p.get(&platform_id))
            .and_then(|e| e.get(&encoding_id))
            .and_then(|l| l.get(&language_id))
            .cloned()
    }

    // ─── getter ──────────────────────────────────────────────────────────────

    /// 返回名称记录列表。
    pub fn name_records(&self) -> &[NameRecord] {
        &self.name_records
    }

    /// 返回字体族名。
    pub fn font_family(&self) -> Option<&str> {
        self.font_family.as_deref()
    }

    /// 返回字体子族名。
    pub fn font_sub_family(&self) -> Option<&str> {
        self.font_sub_family.as_deref()
    }

    /// 返回 PostScript 名称。
    pub fn post_script_name(&self) -> Option<&str> {
        self.post_script_name.as_deref()
    }
}

impl Default for NamingTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let nt = NamingTable::new();
        assert!(nt.name_records().is_empty());
        assert!(nt.font_family().is_none());
    }

    #[test]
    fn test_from_records() {
        let mut nr = NameRecord::with_fields(
            PLATFORM_WINDOWS,
            crate::font::name_record::ENCODING_WINDOWS_UNICODE_BMP,
            LANGUAGE_WINDOWS_EN_US,
            NAME_FONT_FAMILY_NAME,
            0,
            0,
        );
        nr.set_string_value(Some("Arial".to_string()));
        let nt = NamingTable::from_records(vec![nr]);
        assert_eq!(nt.font_family(), Some("Arial"));
    }

    #[test]
    fn test_get_name() {
        let mut nr = NameRecord::with_fields(
            PLATFORM_UNICODE,
            crate::font::name_record::ENCODING_UNICODE_2_0_BMP,
            LANGUAGE_UNICODE,
            NAME_POSTSCRIPT_NAME,
            0,
            0,
        );
        nr.set_string_value(Some("ArialMT".to_string()));
        let nt = NamingTable::from_records(vec![nr]);

        let result = nt.get_name(
            NAME_POSTSCRIPT_NAME,
            PLATFORM_UNICODE,
            crate::font::name_record::ENCODING_UNICODE_2_0_BMP,
            LANGUAGE_UNICODE,
        );
        assert_eq!(result, Some("ArialMT".to_string()));
    }

    #[test]
    fn test_get_name_not_found() {
        let nt = NamingTable::new();
        assert!(nt.get_name(1, 3, 1, 0x0409).is_none());
    }

    #[test]
    fn test_post_script_name_fallback() {
        let mut nr = NameRecord::with_fields(
            PLATFORM_WINDOWS,
            ENCODING_WINDOWS_UNICODE_BMP,
            LANGUAGE_WINDOWS_EN_US,
            NAME_POSTSCRIPT_NAME,
            0,
            0,
        );
        nr.set_string_value(Some("Helvetica-Bold".to_string()));
        let nt = NamingTable::from_records(vec![nr]);
        assert_eq!(nt.post_script_name(), Some("Helvetica-Bold"));
    }

    #[test]
    fn test_default() {
        let nt = NamingTable::default();
        assert!(nt.name_records().is_empty());
    }
}
