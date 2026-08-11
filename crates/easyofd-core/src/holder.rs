//! 值持有者。
//!
//! 对应 Java: org.ofdrw.core.Holder

/// 值持有者。
///
/// 泛型值容器，用于持有和传递单个值。
///
/// 对应 Java: org.ofdrw.core.Holder
#[derive(Debug, Clone, PartialEq)]
pub struct Holder<T> {
    /// 持有的值。
    value: T,
}

impl<T> Holder<T> {
    /// 创建值持有者。
    #[must_use]
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// 获取值的引用。
    #[must_use]
    pub fn get(&self) -> &T {
        &self.value
    }

    /// 获取值的可变引用。
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// 取出值。
    pub fn into_inner(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holder_new() {
        let h = Holder::new(42);
        assert_eq!(*h.get(), 42);
    }

    #[test]
    fn holder_get_mut() {
        let mut h = Holder::new(10);
        *h.get_mut() = 20;
        assert_eq!(*h.get(), 20);
    }

    #[test]
    fn holder_into_inner() {
        let h = Holder::new("test");
        assert_eq!(h.into_inner(), "test");
    }

    #[test]
    fn holder_clone_eq() {
        let h = Holder::new(vec![1, 2, 3]);
        let h2 = h.clone();
        assert_eq!(h, h2);
    }
}
