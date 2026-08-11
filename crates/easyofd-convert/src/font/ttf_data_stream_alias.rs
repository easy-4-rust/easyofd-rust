//! TTF 字节流读取器别名。
//!
//! 对应 Java: org.ofdrw.converter.font.TTFDataStream
//!
//! Java 版 `TTFDataStream` 是 TTF 字体文件的流式读取器。
//! Rust 版复用 [`easyofd_font::TtfDataStream`]，此模块提供
//! Java 类名兼容的类型别名和辅助函数。

/// TTF 字节流读取器（别名）。
///
/// 对应 Java: `org.ofdrw.converter.font.TTFDataStream`
///
/// 内部使用 [`easyofd_font::ttf_data_stream::TtfDataStream`]，提供大端序字节流读取。
pub type TTFDataStream<'a> = easyofd_font::ttf_data_stream::TtfDataStream<'a>;

/// 从字节切片创建 TTF 数据流。
///
/// 对应 Java: `new TTFDataStream(byte[])` 构造器。
///
/// # 参数
/// - `data`: TTF/OTF 字体文件原始字节
#[must_use]
pub fn create_ttf_data_stream(data: &[u8]) -> TTFDataStream<'_> {
    easyofd_font::ttf_data_stream::TtfDataStream::new(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ttf_data_stream() {
        let data = [0x00, 0x01, 0x00, 0x00]; // TTF header start
        let mut stream = create_ttf_data_stream(&data);
        assert_eq!(stream.read_u16(), Some(1));
    }

    #[test]
    fn test_ttf_data_stream_alias_type_check() {
        let data = [0x48, 0x65, 0x6C, 0x6C, 0x6F];
        let mut stream = TTFDataStream::new(&data);
        let bytes = stream.read_bytes(5).unwrap();
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_ttf_data_stream_seek() {
        let data = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut stream = create_ttf_data_stream(&data);
        stream.seek(4);
        assert_eq!(stream.read_u32(), Some(0x0405_0607));
    }
}
