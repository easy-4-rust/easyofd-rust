//! TrueType 字体解析器。
//!
//! 对应 Java 版 `ofdrw-font` 中的 TrueType 字体处理，
//! 解析 TTF 文件的基本结构：偏移表、表目录、`name` 表（字体名称）。

use crate::ttf_data_stream::TtfDataStream;

/// TTF 文件魔数（`0x00010000`）。
pub(crate) const TTF_MAGIC: u32 = 0x0001_0000;
/// OTF 文件魔数（`OTTO`）。
const OTF_MAGIC: u32 = 0x4F54_544F;

/// TrueType 字体解析结果。
///
/// 包含从 TTF 文件中提取的基本元数据：字体名称、家族名称、
/// 表数量等信息。
#[derive(Debug, Clone)]
pub struct TrueTypeFont {
    /// 字体 PostScript 名称或全名。
    font_name: String,
    /// 字体家族名称。
    family_name: String,
    /// 字体表数量。
    num_tables: u16,
    /// 是否为 OTF（CFF）字体。
    is_otf: bool,
}

impl TrueTypeFont {
    /// 从 TTF/OTF 原始字节解析字体。
    ///
    /// # 参数
    /// - `data`：TTF 文件原始字节
    ///
    /// # 错误
    /// 数据格式无效时返回错误消息。
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let mut stream = TtfDataStream::new(data);

        // 读取偏移表
        let sf_version = stream.read_u32().ok_or("TTF 数据不足，无法读取版本号")?;

        let is_otf = sf_version == OTF_MAGIC;
        if sf_version != TTF_MAGIC && !is_otf {
            return Err(format!("无效的 TTF/OTF 版本号: 0x{sf_version:08X}"));
        }

        let num_tables = stream.read_u16().ok_or("TTF 数据不足，无法读取表数量")?;

        // 跳过 searchRange、entrySelector、rangeShift
        stream.read_u16();
        stream.read_u16();
        stream.read_u16();

        // 读取表目录，查找 `name` 表
        let mut name_offset: Option<u32> = None;
        let mut name_length: Option<u32> = None;

        for _ in 0..num_tables {
            let tag = stream.read_tag().ok_or("TTF 数据不足，无法读取表标签")?;
            let _checksum = stream.read_u32().ok_or("TTF 数据不足，无法读取校验和")?;
            let offset = stream.read_u32().ok_or("TTF 数据不足，无法读取表偏移")?;
            let length = stream.read_u32().ok_or("TTF 数据不足，无法读取表长度")?;

            if &tag == b"name" {
                name_offset = Some(offset);
                name_length = Some(length);
            }
        }

        // 解析 `name` 表
        let (font_name, family_name) = if let (Some(off), Some(_len)) = (name_offset, name_length) {
            parse_name_table(data, off as usize)
                .unwrap_or_else(|| ("Unknown".to_string(), "Unknown".to_string()))
        } else {
            ("Unknown".to_string(), "Unknown".to_string())
        };

        Ok(Self {
            font_name,
            family_name,
            num_tables,
            is_otf,
        })
    }

    /// 返回字体名称（PostScript 名称或全名）。
    #[must_use]
    pub fn font_name(&self) -> &str {
        &self.font_name
    }

    /// 返回字体家族名称。
    #[must_use]
    pub fn family_name(&self) -> &str {
        &self.family_name
    }

    /// 返回字体表数量。
    #[must_use]
    pub fn num_tables(&self) -> u16 {
        self.num_tables
    }

    /// 判断是否为 OTF（CFF）字体。
    #[must_use]
    pub fn is_otf(&self) -> bool {
        self.is_otf
    }
}

/// name 表中的平台 ID。
const PLATFORM_ID_UNICODE: u16 = 0;
const PLATFORM_ID_MACINTOSH: u16 = 1;
const PLATFORM_ID_WINDOWS: u16 = 3;

/// name 表中的名称 ID。
const NAME_ID_FONT_FAMILY: u16 = 1;
const NAME_ID_FULL_NAME: u16 = 4;
const NAME_ID_POSTSCRIPT_NAME: u16 = 6;

/// 解析 `name` 表，提取字体名称和家族名称。
fn parse_name_table(data: &[u8], offset: usize) -> Option<(String, String)> {
    let mut stream = TtfDataStream::new(data);
    stream.seek(offset);

    let _format = stream.read_u16()?;
    let count = stream.read_u16()?;
    let string_offset = stream.read_u16()? as usize;

    let mut family_name = String::new();
    let mut full_name = String::new();
    let mut postscript_name = String::new();

    for _ in 0..count {
        let platform_id = stream.read_u16()?;
        let encoding_id = stream.read_u16()?;
        let _language_id = stream.read_u16()?;
        let name_id = stream.read_u16()?;
        let length = stream.read_u16()? as usize;
        let name_record_offset = stream.read_u16()? as usize;

        let abs_offset = offset + string_offset + name_record_offset;
        if abs_offset + length > data.len() {
            continue;
        }

        let name_bytes = &data[abs_offset..abs_offset + length];
        let name_str = decode_name(platform_id, encoding_id, name_bytes);

        match name_id {
            NAME_ID_FONT_FAMILY if family_name.is_empty() || platform_id == PLATFORM_ID_WINDOWS => {
                family_name.clone_from(&name_str);
            }
            NAME_ID_FULL_NAME if full_name.is_empty() || platform_id == PLATFORM_ID_WINDOWS => {
                full_name.clone_from(&name_str);
            }
            NAME_ID_POSTSCRIPT_NAME if postscript_name.is_empty() => {
                postscript_name = name_str;
            }
            _ => {}
        }
    }

    let font_name = if !postscript_name.is_empty() {
        postscript_name
    } else if !full_name.is_empty() {
        full_name
    } else {
        "Unknown".to_string()
    };

    let family = if family_name.is_empty() {
        "Unknown".to_string()
    } else {
        family_name
    };

    Some((font_name, family))
}

/// 根据平台 ID 和编码 ID 解码名称字节。
fn decode_name(platform_id: u16, _encoding_id: u16, bytes: &[u8]) -> String {
    match platform_id {
        PLATFORM_ID_UNICODE | PLATFORM_ID_WINDOWS => {
            // UTF-16 BE
            let mut chars = Vec::new();
            let mut i = 0;
            while i + 1 < bytes.len() {
                let code = u16::from_be_bytes([bytes[i], bytes[i + 1]]);
                chars.push(code);
                i += 2;
            }
            String::from_utf16_lossy(&chars)
        }
        PLATFORM_ID_MACINTOSH => {
            // MacRoman 编码，简化处理：直接尝试 UTF-8
            String::from_utf8_lossy(bytes).to_string()
        }
        _ => String::from_utf8_lossy(bytes).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个最小的有效 TTF 文件字节。
    fn build_minimal_ttf() -> Vec<u8> {
        let mut data = Vec::new();

        // 偏移表 (12 bytes)
        data.extend_from_slice(&TTF_MAGIC.to_be_bytes()); // sfVersion
        data.extend_from_slice(&1u16.to_be_bytes()); // numTables
        data.extend_from_slice(&0u16.to_be_bytes()); // searchRange
        data.extend_from_slice(&0u16.to_be_bytes()); // entrySelector
        data.extend_from_slice(&0u16.to_be_bytes()); // rangeShift

        // 表目录条目 (16 bytes)
        data.extend_from_slice(b"name"); // tag
        data.extend_from_slice(&0u32.to_be_bytes()); // checksum
        let name_offset = 12u32 + 16; // 偏移表 + 1个表目录条目
        data.extend_from_slice(&name_offset.to_be_bytes()); // offset
        data.extend_from_slice(&0u32.to_be_bytes()); // length (稍后更新)

        // name 表
        data.extend_from_slice(&0u16.to_be_bytes()); // format
        data.extend_from_slice(&0u16.to_be_bytes()); // count
        data.extend_from_slice(&0u16.to_be_bytes()); // stringOffset

        // 更新 name 表长度
        #[allow(clippy::cast_possible_truncation)]
        let name_len = (data.len() - name_offset as usize) as u32;
        let len_pos = 12 + 8 + 4 + 4; // 跳过 offset 字段
        data[len_pos..len_pos + 4].copy_from_slice(&name_len.to_be_bytes());

        data
    }

    #[test]
    fn test_parse_minimal_ttf() {
        let data = build_minimal_ttf();
        let font = TrueTypeFont::parse(&data).unwrap();
        assert_eq!(font.num_tables(), 1);
        assert!(!font.is_otf());
        assert_eq!(font.font_name(), "Unknown");
        assert_eq!(font.family_name(), "Unknown");
    }

    #[test]
    fn test_parse_invalid_data() {
        let data = [0x00, 0x00, 0x00, 0x00];
        let result = TrueTypeFont::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_too_short() {
        let data = [0x00, 0x01];
        let result = TrueTypeFont::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_otf_magic() {
        let mut data = Vec::new();
        data.extend_from_slice(&OTF_MAGIC.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        let font = TrueTypeFont::parse(&data).unwrap();
        assert!(font.is_otf());
    }
}
