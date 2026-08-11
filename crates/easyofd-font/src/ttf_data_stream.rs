//! TTF 字节流读取器。
//!
//! 提供大端序（Big-Endian）字节流读取能力，用于解析 TrueType / OpenType 字体文件的二进制数据。

/// TTF 字节流读取器。
///
/// 以大端序从字节切片中读取 `u8`、`u16`、`i16`、`u32`、`i32` 等基本类型，
/// 支持定位到指定偏移量。
#[derive(Debug, Clone)]
pub struct TtfDataStream<'a> {
    /// 底层字节数据。
    data: &'a [u8],
    /// 当前读取位置。
    pos: usize,
}

impl<'a> TtfDataStream<'a> {
    /// 从字节切片创建流读取器。
    ///
    /// # 参数
    /// - `data`：字体文件原始字节
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// 返回当前读取位置。
    #[must_use]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// 定位到指定偏移量。
    ///
    /// # 参数
    /// - `offset`：目标偏移量（字节）
    pub fn seek(&mut self, offset: usize) {
        self.pos = offset;
    }

    /// 返回底层数据的总长度。
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 判断底层数据是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 返回剩余可读字节数。
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// 读取一个 `u8`。
    ///
    /// # 错误
    /// 当数据不足时返回 `None`。
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() {
            return None;
        }
        let val = self.data[self.pos];
        self.pos += 1;
        Some(val)
    }

    /// 读取一个大端序 `u16`。
    pub fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 > self.data.len() {
            return None;
        }
        let val = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Some(val)
    }

    /// 读取一个大端序 `i16`。
    pub fn read_i16(&mut self) -> Option<i16> {
        if self.pos + 2 > self.data.len() {
            return None;
        }
        let val = i16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Some(val)
    }

    /// 读取一个大端序 `u32`。
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 > self.data.len() {
            return None;
        }
        let val = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Some(val)
    }

    /// 读取一个大端序 `i32`。
    pub fn read_i32(&mut self) -> Option<i32> {
        if self.pos + 4 > self.data.len() {
            return None;
        }
        let val = i32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Some(val)
    }

    /// 读取指定长度的字节切片。
    ///
    /// 返回对底层数据的引用（零拷贝）。
    pub fn read_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        if self.pos + len > self.data.len() {
            return None;
        }
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Some(slice)
    }

    /// 读取一个标签（4 字节 ASCII）。
    ///
    /// 用于读取 TTF 表标签（如 `head`、`name`、`cmap` 等）。
    pub fn read_tag(&mut self) -> Option<[u8; 4]> {
        let bytes = self.read_bytes(4)?;
        let mut tag = [0u8; 4];
        tag.copy_from_slice(bytes);
        Some(tag)
    }

    /// 查看当前位置的 `u32` 但不移动位置。
    #[must_use]
    pub fn peek_u32(&self) -> Option<u32> {
        if self.pos + 4 > self.data.len() {
            return None;
        }
        Some(u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]))
    }

    /// 返回底层数据的引用。
    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.data
    }

    /// 返回从当前位置到数据末尾的子切片。
    #[must_use]
    pub fn remaining_bytes(&self) -> &'a [u8] {
        if self.pos >= self.data.len() {
            &[]
        } else {
            &self.data[self.pos..]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x01, 0xFF, 0x80];
        let mut stream = TtfDataStream::new(&data);
        assert_eq!(stream.read_u8(), Some(0x01));
        assert_eq!(stream.read_u8(), Some(0xFF));
        assert_eq!(stream.read_u8(), Some(0x80));
        assert_eq!(stream.read_u8(), None);
    }

    #[test]
    fn test_read_u16_be() {
        let data = [0x00, 0x01, 0xFF, 0xFF];
        let mut stream = TtfDataStream::new(&data);
        assert_eq!(stream.read_u16(), Some(1));
        assert_eq!(stream.read_u16(), Some(0xFFFF));
    }

    #[test]
    fn test_read_i16_be() {
        let data = [0xFF, 0xFF, 0x00, 0x01];
        let mut stream = TtfDataStream::new(&data);
        assert_eq!(stream.read_i16(), Some(-1));
        assert_eq!(stream.read_i16(), Some(1));
    }

    #[test]
    fn test_read_u32_be() {
        let data = [0x00, 0x01, 0x00, 0x00];
        let mut stream = TtfDataStream::new(&data);
        assert_eq!(stream.read_u32(), Some(0x0001_0000));
    }

    #[test]
    fn test_read_i32_be() {
        let data = [0xFF, 0xFF, 0xFF, 0xFF];
        let mut stream = TtfDataStream::new(&data);
        assert_eq!(stream.read_i32(), Some(-1));
    }

    #[test]
    fn test_read_bytes() {
        let data = [0x48, 0x65, 0x6C, 0x6C, 0x6F];
        let mut stream = TtfDataStream::new(&data);
        let bytes = stream.read_bytes(5).unwrap();
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_read_tag() {
        let data = *b"headnamecmap";
        let mut stream = TtfDataStream::new(&data);
        assert_eq!(stream.read_tag(), Some(*b"head"));
        assert_eq!(stream.read_tag(), Some(*b"name"));
        assert_eq!(stream.read_tag(), Some(*b"cmap"));
    }

    #[test]
    fn test_seek_and_position() {
        let data = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut stream = TtfDataStream::new(&data);
        stream.seek(4);
        assert_eq!(stream.position(), 4);
        assert_eq!(stream.read_u32(), Some(0x0405_0607));
    }

    #[test]
    fn test_peek_u32() {
        let data = [0x00, 0x00, 0x00, 0x01];
        let stream = TtfDataStream::new(&data);
        assert_eq!(stream.peek_u32(), Some(1));
        // 位置不变
        assert_eq!(stream.position(), 0);
    }

    #[test]
    fn test_len_is_empty() {
        let data = [0u8; 10];
        let stream = TtfDataStream::new(&data);
        assert_eq!(stream.len(), 10);
        assert!(!stream.is_empty());

        let empty = TtfDataStream::new(&[]);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_remaining() {
        let data = [0u8; 8];
        let mut stream = TtfDataStream::new(&data);
        assert_eq!(stream.remaining(), 8);
        stream.read_u32();
        assert_eq!(stream.remaining(), 4);
    }

    #[test]
    fn test_read_insufficient_data() {
        let data = [0x00];
        let mut stream = TtfDataStream::new(&data);
        assert_eq!(stream.read_u16(), None);
        assert_eq!(stream.read_u32(), None);
    }

    #[test]
    fn test_remaining_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut stream = TtfDataStream::new(&data);
        stream.seek(2);
        assert_eq!(stream.remaining_bytes(), &[3, 4, 5]);
    }

    #[test]
    fn test_as_bytes() {
        let data = [10, 20, 30];
        let stream = TtfDataStream::new(&data);
        assert_eq!(stream.as_bytes(), &[10, 20, 30]);
    }
}
