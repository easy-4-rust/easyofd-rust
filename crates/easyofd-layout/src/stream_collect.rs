//! 流式收集器。
//!
//! 对应 Java: org.ofdrw.layout.StreamCollect

/// 流式收集器，用于收集布局过程中的流式输出。
///
/// 对应 Java: ofdrw layout StreamCollect。
#[derive(Debug, Clone, Default)]
pub struct StreamCollect {
    /// 收集的数据。
    pub data: Vec<u8>,
    /// 是否已完成收集。
    pub finished: bool,
}

impl StreamCollect {
    /// 创建流式收集器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 写入数据。
    pub fn write(&mut self, buf: &[u8]) {
        self.data.extend_from_slice(buf);
    }

    /// 标记收集完成。
    pub fn finish(&mut self) {
        self.finished = true;
    }

    /// 获取收集的数据长度。
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 是否已完成。
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sc = StreamCollect::new();
        assert!(sc.is_empty());
        assert!(!sc.is_finished());
    }

    #[test]
    fn test_write() {
        let mut sc = StreamCollect::new();
        sc.write(b"hello");
        sc.write(b" world");
        assert_eq!(sc.len(), 11);
        assert!(!sc.is_empty());
    }

    #[test]
    fn test_finish() {
        let mut sc = StreamCollect::new();
        sc.write(b"data");
        sc.finish();
        assert!(sc.is_finished());
        assert_eq!(sc.len(), 4);
    }
}
