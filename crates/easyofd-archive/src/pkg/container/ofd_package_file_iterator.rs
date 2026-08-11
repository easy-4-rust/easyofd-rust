//! OFD 包文件迭代器接口。
//!
//! 对应 Java: org.ofdrw.pkg.container.OFDPackageFileIterator
//!
//! 定义遍历 OFD 包中所有文件的迭代器接口。

/// OFD 包文件迭代器接口。
///
/// 对应 Java: `org.ofdrw.pkg.container.OFDPackageFileIterator`
///
/// 用于遍历 OFD 包中的所有文件条目。
/// 实现者需要提供文件路径和内容的迭代能力。
pub trait OfdPackageFileIterator: Send + Sync {
    /// 获取下一个文件条目。
    ///
    /// 返回 `Some((路径, 内容))` 或 `None` 表示遍历结束。
    fn next_file(&mut self) -> Option<(String, Vec<u8>)>;

    /// 重置迭代器到初始位置。
    fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockIterator {
        items: Vec<(String, Vec<u8>)>,
        index: usize,
    }

    impl MockIterator {
        fn new(items: Vec<(String, Vec<u8>)>) -> Self {
            Self { items, index: 0 }
        }
    }

    impl OfdPackageFileIterator for MockIterator {
        fn next_file(&mut self) -> Option<(String, Vec<u8>)> {
            if self.index < self.items.len() {
                let item = self.items[self.index].clone();
                self.index += 1;
                Some(item)
            } else {
                None
            }
        }

        fn reset(&mut self) {
            self.index = 0;
        }
    }

    #[test]
    fn iterator_basic() {
        let items = vec![
            ("OFD.xml".to_string(), b"<OFD/>".to_vec()),
            ("Doc.xml".to_string(), b"<Doc/>".to_vec()),
        ];
        let mut iter = MockIterator::new(items);

        assert_eq!(iter.next_file().unwrap().0, "OFD.xml");
        assert_eq!(iter.next_file().unwrap().0, "Doc.xml");
        assert!(iter.next_file().is_none());
    }

    #[test]
    fn iterator_reset() {
        let items = vec![("a.xml".to_string(), b"<a/>".to_vec())];
        let mut iter = MockIterator::new(items);

        assert!(iter.next_file().is_some());
        assert!(iter.next_file().is_none());

        iter.reset();
        assert!(iter.next_file().is_some());
    }
}
