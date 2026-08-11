//! 可选值类型。
//!
//! 对应 Java: org.ofdrw.core.graph.pathObj.OptVal

/// 可选值类型。
///
/// 对应 Java: org.ofdrw.core.graph.pathObj.OptVal
#[derive(Debug, Clone, PartialEq)]
pub struct OptVal<T> {
    /// 值。
    pub value: Option<T>,
}

impl<T> OptVal<T> {
    /// 创建空值。
    #[must_use]
    pub fn none() -> Self {
        Self { value: None }
    }

    /// 创建有值。
    #[must_use]
    pub fn some(value: T) -> Self {
        Self { value: Some(value) }
    }

    /// 是否有值。
    #[must_use]
    pub fn is_some(&self) -> bool {
        self.value.is_some()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.value.is_none()
    }

    /// 获取值的引用。
    #[must_use]
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

impl<T: Default> Default for OptVal<T> {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_val_none() {
        let v: OptVal<i32> = OptVal::none();
        assert!(v.is_none());
        assert!(!v.is_some());
    }

    #[test]
    fn opt_val_some() {
        let v = OptVal::some(42);
        assert!(v.is_some());
        assert_eq!(v.get(), Some(&42));
    }

    #[test]
    fn opt_val_default() {
        let v: OptVal<i32> = OptVal::default();
        assert!(v.is_none());
    }
}
