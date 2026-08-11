//! 内存 TTF 数据流。
//!
//! 对应 Java: org.ofdrw.converter.font.MemoryTTFDataStream
//!
//! Java 版 `MemoryTTFDataStream` 将 TTF 字体数据加载到内存中，
//! 提供与 `TTFDataStream` 相同的流式读取接口，但数据由自身拥有。
//! Rust 版通过持有 `Vec<u8>` 实现所有权语义。

use easyofd_font::ttf_data_stream::TtfDataStream;

/// 内存 TTF 数据流。
///
/// 对应 Java: `org.ofdrw.converter.font.MemoryTTFDataStream`
///
/// 将字体数据完全加载到内存，拥有数据所有权。
/// 通过 [`as_stream`] 方法获取 [`TtfDataStream`] 进行读取。
#[derive(Debug, Clone)]
pub struct MemoryTTFDataStream {
    /// 内部持有的字体数据。
    data: Vec<u8>,
}

impl MemoryTTFDataStream {
    /// 从字节数组创建内存数据流。
    ///
    /// 对应 Java: `MemoryTTFDataStream(byte[] data)` 构造器。
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// 从文件路径加载字体数据。
    ///
    /// 对应 Java: `MemoryTTFDataStream(Path path)` 构造器。
    ///
    /// # 错误
    /// 文件读取失败时返回 `std::io::Error`。
    pub fn from_path(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        Ok(Self { data })
    }

    /// 获取底层数据的引用。
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 获取底层数据的长度（字节）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 判断底层数据是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 创建 [`TtfDataStream`] 进行流式读取。
    ///
    /// 返回的流引用此结构体的内部数据，生命周期与调用者绑定。
    #[must_use]
    pub fn as_stream(&self) -> TtfDataStream<'_> {
        TtfDataStream::new(&self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let data = vec![0x00, 0x01, 0x00, 0x00];
        let mts = MemoryTTFDataStream::new(data.clone());
        assert_eq!(mts.data(), &data);
        assert_eq!(mts.len(), 4);
        assert!(!mts.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let mts = MemoryTTFDataStream::new(vec![]);
        assert!(mts.is_empty());
        assert_eq!(mts.len(), 0);
    }

    #[test]
    fn test_as_stream() {
        let data = vec![0x00, 0x01, 0x00, 0x02];
        let mts = MemoryTTFDataStream::new(data);
        let mut stream = mts.as_stream();
        assert_eq!(stream.read_u16(), Some(1));
        assert_eq!(stream.read_u16(), Some(2));
    }

    #[test]
    fn test_as_stream_reusable() {
        let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
        let mts = MemoryTTFDataStream::new(data);

        // 可以多次创建流
        let mut s1 = mts.as_stream();
        assert_eq!(s1.read_u32(), Some(0xAABB_CCDD));

        let mut s2 = mts.as_stream();
        assert_eq!(s2.read_u32(), Some(0xAABB_CCDD));
    }

    #[test]
    fn test_clone() {
        let mts1 = MemoryTTFDataStream::new(vec![1, 2, 3]);
        let mts2 = mts1.clone();
        assert_eq!(mts1.data(), mts2.data());
    }

    #[test]
    fn test_from_path_nonexistent() {
        let result = MemoryTTFDataStream::from_path("/nonexistent/font.ttf");
        assert!(result.is_err());
    }
}
