//! Type1 字体分段解析器。
//!
//! 对应 Java: org.ofdrw.converter.font.type1.Type1SegSplitParser
//!
//! Java 版用于解析 Type1 字体文件的分段结构（PFB 格式的 ASCII 段和二进制段）。
//! Rust 版提供简化的段信息解析。

/// Type1 字体段类型。
///
/// 对应 Java PFB 段标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Type1SegType {
    /// ASCII 段（PostScript 文本）。
    Ascii,
    /// 二进制段（加密的 CharStrings 等数据）。
    Binary,
    /// 结束标记段。
    End,
}

/// Type1 字体文件段。
///
/// 对应 Java: `Type1SegSplitParser` 解析出的段结构。
#[derive(Debug, Clone)]
pub struct Type1Seg {
    /// 段类型。
    pub seg_type: Type1SegType,
    /// 段数据。
    pub data: Vec<u8>,
}

/// Type1 字体分段解析器。
///
/// 对应 Java: `org.ofdrw.converter.font.type1.Type1SegSplitParser`
///
/// 解析 PFB（Printer Font Binary）格式的 Type1 字体文件，
/// 将其拆分为 ASCII 段和二进制段。
#[derive(Debug)]
pub struct Type1SegSplitParser;

impl Type1SegSplitParser {
    /// 解析 PFB 格式的 Type1 字体数据。
    ///
    /// PFB 文件由多个段组成，每段以 0x80 开头，后跟段类型标识：
    /// - `0x01`: ASCII 段
    /// - `0x02`: 二进制段
    /// - `0x03`: 结束标记
    ///
    /// # 参数
    /// - `data`: PFB 格式的字体文件原始字节
    ///
    /// # 返回
    /// 解析出的段列表。如果数据格式无效，返回空列表。
    #[must_use]
    pub fn parse(data: &[u8]) -> Vec<Type1Seg> {
        let mut segments = Vec::new();
        let mut pos = 0;

        while pos + 6 <= data.len() {
            // PFB 段头: 0x80 + type(1) + length(4, little-endian)
            if data[pos] != 0x80 {
                break;
            }
            let seg_type = data[pos + 1];
            let length =
                u32::from_le_bytes([data[pos + 2], data[pos + 3], data[pos + 4], data[pos + 5]])
                    as usize;

            pos += 6;
            if pos + length > data.len() {
                break;
            }

            let seg = match seg_type {
                0x01 => Type1Seg {
                    seg_type: Type1SegType::Ascii,
                    data: data[pos..pos + length].to_vec(),
                },
                0x02 => Type1Seg {
                    seg_type: Type1SegType::Binary,
                    data: data[pos..pos + length].to_vec(),
                },
                0x03 => Type1Seg {
                    seg_type: Type1SegType::End,
                    data: Vec::new(),
                },
                _ => break,
            };

            segments.push(seg);
            pos += length;

            if seg_type == 0x03 {
                break;
            }
        }

        segments
    }

    /// 判断数据是否为 PFB 格式。
    #[must_use]
    pub fn is_pfb(data: &[u8]) -> bool {
        data.len() >= 6 && data[0] == 0x80 && (data[1] == 0x01 || data[1] == 0x02)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pfb_valid() {
        // PFB header: 0x80, 0x01 (ASCII), length=5
        let data = [
            0x80, 0x01, 0x05, 0x00, 0x00, 0x00, b'h', b'e', b'l', b'l', b'o',
        ];
        assert!(Type1SegSplitParser::is_pfb(&data));
    }

    #[test]
    fn test_is_pfb_invalid() {
        assert!(!Type1SegSplitParser::is_pfb(&[]));
        assert!(!Type1SegSplitParser::is_pfb(&[0x00, 0x01]));
        assert!(!Type1SegSplitParser::is_pfb(&[0x80, 0x04]));
    }

    #[test]
    fn test_parse_single_ascii_segment() {
        let mut data = vec![0x80, 0x01, 0x05, 0x00, 0x00, 0x00]; // ASCII, 5 bytes
        data.extend_from_slice(b"hello");

        let segs = Type1SegSplitParser::parse(&data);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].seg_type, Type1SegType::Ascii);
        assert_eq!(segs[0].data, b"hello");
    }

    #[test]
    fn test_parse_multiple_segments() {
        let mut data = Vec::new();
        // ASCII segment
        data.push(0x80);
        data.push(0x01);
        data.extend_from_slice(&3u32.to_le_bytes());
        data.extend_from_slice(b"abc");
        // Binary segment
        data.push(0x80);
        data.push(0x02);
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&[0xFF, 0xFE]);
        // End segment
        data.push(0x80);
        data.push(0x03);
        data.extend_from_slice(&0u32.to_le_bytes());

        let segs = Type1SegSplitParser::parse(&data);
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].seg_type, Type1SegType::Ascii);
        assert_eq!(segs[1].seg_type, Type1SegType::Binary);
        assert_eq!(segs[2].seg_type, Type1SegType::End);
    }

    #[test]
    fn test_parse_empty_data() {
        assert!(Type1SegSplitParser::parse(&[]).is_empty());
    }

    #[test]
    fn test_parse_truncated() {
        // 声称有 10 字节但实际不够
        let data = [0x80, 0x01, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x02];
        assert!(Type1SegSplitParser::parse(&data).is_empty());
    }
}
