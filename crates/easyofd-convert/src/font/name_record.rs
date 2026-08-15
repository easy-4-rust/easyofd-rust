//! 字体名称记录。
//!
//! 对应 Java: org.ofdrw.converter.font.NameRecord
//!
//! 参考 Apache FontBox `NameRecord`，遵循 OpenType `name` 表规范。

/// 平台 ID：Unicode。
pub const PLATFORM_UNICODE: u16 = 0;
/// 平台 ID：Macintosh。
pub const PLATFORM_MACINTOSH: u16 = 1;
/// 平台 ID：ISO（已废弃）。
pub const PLATFORM_ISO: u16 = 2;
/// 平台 ID：Windows。
pub const PLATFORM_WINDOWS: u16 = 3;

// ─── Unicode 编码 ID ─────────────────────────────────────────────────────────

/// Unicode 编码 ID：1.0。
pub const ENCODING_UNICODE_1_0: u16 = 0;
/// Unicode 编码 ID：1.1。
pub const ENCODING_UNICODE_1_1: u16 = 1;
/// Unicode 编码 ID：2.0 BMP。
pub const ENCODING_UNICODE_2_0_BMP: u16 = 3;
/// Unicode 编码 ID：2.0 Full。
pub const ENCODING_UNICODE_2_0_FULL: u16 = 4;

/// Unicode 语言 ID。
pub const LANGUAGE_UNICODE: u16 = 0;

// ─── Windows 编码 ID ─────────────────────────────────────────────────────────

/// Windows 编码 ID：Symbol。
pub const ENCODING_WINDOWS_SYMBOL: u16 = 0;
/// Windows 编码 ID：Unicode BMP。
pub const ENCODING_WINDOWS_UNICODE_BMP: u16 = 1;
/// Windows 编码 ID：Unicode UCS-4。
pub const ENCODING_WINDOWS_UNICODE_UCS4: u16 = 10;

/// Windows 语言 ID：en-US。
pub const LANGUAGE_WINDOWS_EN_US: u16 = 0x0409;

// ─── Macintosh 编码 ID ───────────────────────────────────────────────────────

/// Macintosh 编码 ID：Roman。
pub const ENCODING_MACINTOSH_ROMAN: u16 = 0;

/// Macintosh 语言 ID：English。
pub const LANGUAGE_MACINTOSH_ENGLISH: u16 = 0;

// ─── 名称 ID ─────────────────────────────────────────────────────────────────

/// 名称 ID：版权信息。
pub const NAME_COPYRIGHT: u16 = 0;
/// 名称 ID：字体族名。
pub const NAME_FONT_FAMILY_NAME: u16 = 1;
/// 名称 ID：字体子族名。
pub const NAME_FONT_SUB_FAMILY_NAME: u16 = 2;
/// 名称 ID：唯一字体 ID。
pub const NAME_UNIQUE_FONT_ID: u16 = 3;
/// 名称 ID：完整字体名。
pub const NAME_FULL_FONT_NAME: u16 = 4;
/// 名称 ID：版本。
pub const NAME_VERSION: u16 = 5;
/// 名称 ID：PostScript 名称。
pub const NAME_POSTSCRIPT_NAME: u16 = 6;
/// 名称 ID：商标。
pub const NAME_TRADEMARK: u16 = 7;

/// OpenType `name` 表中的一条名称记录。
///
/// 对应 Java `NameRecord`。每条记录由四元组
/// `(platform_id, encoding_id, language_id, name_id)` 唯一标识，
/// 指向一个字符串值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameRecord {
    /// 平台 ID。
    platform_id: u16,
    /// 平台编码 ID。
    platform_encoding_id: u16,
    /// 语言 ID。
    language_id: u16,
    /// 名称 ID。
    name_id: u16,
    /// 字符串数据长度（字节）。
    string_length: u16,
    /// 字符串数据偏移量（相对于字符串存储区起始）。
    string_offset: u16,
    /// 解析后的字符串值。
    string_value: Option<String>,
}

impl NameRecord {
    /// 创建空的名称记录。
    pub fn new() -> Self {
        Self {
            platform_id: 0,
            platform_encoding_id: 0,
            language_id: 0,
            name_id: 0,
            string_length: 0,
            string_offset: 0,
            string_value: None,
        }
    }

    /// 从原始字段创建名称记录。
    pub fn with_fields(
        platform_id: u16,
        platform_encoding_id: u16,
        language_id: u16,
        name_id: u16,
        string_length: u16,
        string_offset: u16,
    ) -> Self {
        Self {
            platform_id,
            platform_encoding_id,
            language_id,
            name_id,
            string_length,
            string_offset,
            string_value: None,
        }
    }

    /// 从字节流读取记录头部（6 个 u16 字段）。
    ///
    /// 返回 `None` 如果数据不足。
    pub fn read_header(data: &[u8], offset: usize) -> Option<Self> {
        if offset + 12 > data.len() {
            return None;
        }
        let platform_id = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let platform_encoding_id = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
        let language_id = u16::from_be_bytes([data[offset + 4], data[offset + 5]]);
        let name_id = u16::from_be_bytes([data[offset + 6], data[offset + 7]]);
        let string_length = u16::from_be_bytes([data[offset + 8], data[offset + 9]]);
        let string_offset = u16::from_be_bytes([data[offset + 10], data[offset + 11]]);

        Some(Self::with_fields(
            platform_id,
            platform_encoding_id,
            language_id,
            name_id,
            string_length,
            string_offset,
        ))
    }

    // ─── getter/setter ───────────────────────────────────────────────────────

    /// 返回平台 ID。
    pub fn platform_id(&self) -> u16 {
        self.platform_id
    }
    /// 设置平台 ID。
    pub fn set_platform_id(&mut self, v: u16) {
        self.platform_id = v;
    }

    /// 返回平台编码 ID。
    pub fn platform_encoding_id(&self) -> u16 {
        self.platform_encoding_id
    }
    /// 设置平台编码 ID。
    pub fn set_platform_encoding_id(&mut self, v: u16) {
        self.platform_encoding_id = v;
    }

    /// 返回语言 ID。
    pub fn language_id(&self) -> u16 {
        self.language_id
    }
    /// 设置语言 ID。
    pub fn set_language_id(&mut self, v: u16) {
        self.language_id = v;
    }

    /// 返回名称 ID。
    pub fn name_id(&self) -> u16 {
        self.name_id
    }
    /// 设置名称 ID。
    pub fn set_name_id(&mut self, v: u16) {
        self.name_id = v;
    }

    /// 返回字符串长度。
    pub fn string_length(&self) -> u16 {
        self.string_length
    }
    /// 设置字符串长度。
    pub fn set_string_length(&mut self, v: u16) {
        self.string_length = v;
    }

    /// 返回字符串偏移量。
    pub fn string_offset(&self) -> u16 {
        self.string_offset
    }
    /// 设置字符串偏移量。
    pub fn set_string_offset(&mut self, v: u16) {
        self.string_offset = v;
    }

    /// 返回名称字符串值。
    pub fn string_value(&self) -> Option<&str> {
        self.string_value.as_deref()
    }
    /// 设置名称字符串值。
    pub fn set_string_value(&mut self, v: Option<String>) {
        self.string_value = v;
    }
}

impl Default for NameRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NameRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "platform={} pEncoding={} language={} name={} {:?}",
            self.platform_id,
            self.platform_encoding_id,
            self.language_id,
            self.name_id,
            self.string_value
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let nr = NameRecord::new();
        assert_eq!(nr.platform_id(), 0);
        assert_eq!(nr.name_id(), 0);
        assert!(nr.string_value().is_none());
    }

    #[test]
    fn test_with_fields() {
        let nr = NameRecord::with_fields(3, 1, 0x0409, 1, 20, 0);
        assert_eq!(nr.platform_id(), PLATFORM_WINDOWS);
        assert_eq!(nr.platform_encoding_id(), ENCODING_WINDOWS_UNICODE_BMP);
        assert_eq!(nr.language_id(), LANGUAGE_WINDOWS_EN_US);
        assert_eq!(nr.name_id(), NAME_FONT_FAMILY_NAME);
        assert_eq!(nr.string_length(), 20);
    }

    #[test]
    fn test_setters() {
        let mut nr = NameRecord::new();
        nr.set_platform_id(3);
        nr.set_name_id(4);
        nr.set_string_value(Some("TestFont".to_string()));
        assert_eq!(nr.platform_id(), 3);
        assert_eq!(nr.name_id(), 4);
        assert_eq!(nr.string_value(), Some("TestFont"));
    }

    #[test]
    fn test_read_header() {
        // 构造 12 字节数据：platform=3, encoding=1, language=0x0409, name=1, length=10, offset=0
        let data: Vec<u8> = vec![
            0x00, 0x03, // platform_id = 3
            0x00, 0x01, // encoding_id = 1
            0x04, 0x09, // language_id = 0x0409
            0x00, 0x01, // name_id = 1
            0x00, 0x0A, // string_length = 10
            0x00, 0x00, // string_offset = 0
        ];
        let nr = NameRecord::read_header(&data, 0).unwrap();
        assert_eq!(nr.platform_id(), 3);
        assert_eq!(nr.platform_encoding_id(), 1);
        assert_eq!(nr.language_id(), 0x0409);
        assert_eq!(nr.name_id(), 1);
        assert_eq!(nr.string_length(), 10);
        assert_eq!(nr.string_offset(), 0);
    }

    #[test]
    fn test_read_header_insufficient() {
        let data = vec![0u8; 5];
        assert!(NameRecord::read_header(&data, 0).is_none());
    }

    #[test]
    fn test_display() {
        let nr = NameRecord::with_fields(3, 1, 0x0409, 1, 0, 0);
        let display = format!("{nr}");
        assert!(display.contains("platform=3"));
        assert!(display.contains("name=1"));
    }

    #[test]
    fn test_default() {
        let nr = NameRecord::default();
        assert_eq!(nr, NameRecord::new());
    }

    #[test]
    fn test_constants() {
        assert_eq!(PLATFORM_UNICODE, 0);
        assert_eq!(PLATFORM_MACINTOSH, 1);
        assert_eq!(PLATFORM_WINDOWS, 3);
        assert_eq!(NAME_POSTSCRIPT_NAME, 6);
        assert_eq!(LANGUAGE_WINDOWS_EN_US, 0x0409);
    }
}
