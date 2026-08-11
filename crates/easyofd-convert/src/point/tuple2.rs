//! 通用二元组。
//!
//! 对应 Java: org.ofdrw.converter.point.Tuple2

/// 通用二元组，持有两个不同类型的值。
///
/// 对应 Java `Tuple2<X, Y>`，用于在不引入第三方 crate 的情况下返回两个值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tuple2<X, Y> {
    /// 第一个值。
    first: X,
    /// 第二个值。
    second: Y,
}

impl<X, Y> Tuple2<X, Y> {
    /// 创建二元组。
    ///
    /// # 参数
    /// - `first`：第一个值
    /// - `second`：第二个值
    pub fn new(first: X, second: Y) -> Self {
        Self { first, second }
    }

    /// 返回第一个值的引用。
    pub fn first(&self) -> &X {
        &self.first
    }

    /// 返回第二个值的引用。
    pub fn second(&self) -> &Y {
        &self.second
    }

    /// 返回第一个值的可变引用。
    pub fn first_mut(&mut self) -> &mut X {
        &mut self.first
    }

    /// 返回第二个值的可变引用。
    pub fn second_mut(&mut self) -> &mut Y {
        &mut self.second
    }

    /// 消耗二元组，返回 `(first, second)` 元组。
    pub fn into_tuple(self) -> (X, Y) {
        (self.first, self.second)
    }
}

impl<X, Y> From<(X, Y)> for Tuple2<X, Y> {
    fn from(tuple: (X, Y)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_accessors() {
        let t = Tuple2::new(1, "hello");
        assert_eq!(*t.first(), 1);
        assert_eq!(*t.second(), "hello");
    }

    #[test]
    fn test_mutable_accessors() {
        let mut t = Tuple2::new(10, 20.0);
        *t.first_mut() = 30;
        *t.second_mut() = 40.0;
        assert_eq!(*t.first(), 30);
        assert_eq!(*t.second(), 40.0);
    }

    #[test]
    fn test_into_tuple() {
        let t = Tuple2::new("a", 42);
        assert_eq!(t.into_tuple(), ("a", 42));
    }

    #[test]
    fn test_from_tuple() {
        let t: Tuple2<i32, f64> = (5, 3.14).into();
        assert_eq!(*t.first(), 5);
        assert!((t.second() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_clone_and_eq() {
        let t1 = Tuple2::new(1, 2);
        let t2 = t1;
        assert_eq!(t1, t2);
    }
}
