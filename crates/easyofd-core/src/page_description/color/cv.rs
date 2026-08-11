//! 颜色分量值。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.colorSpace.CV

/// 颜色分量值。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.colorSpace.CV
#[derive(Debug, Clone, PartialEq)]
pub struct CV(pub Vec<f64>);

impl CV {
    /// 创建颜色分量值。
    #[must_use]
    pub fn new(values: Vec<f64>) -> Self {
        Self(values)
    }

    /// 获取分量数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 获取指定索引的分量值。
    #[must_use]
    pub fn get(&self, index: usize) -> Option<f64> {
        self.0.get(index).copied()
    }

    /// 序列化为空格分隔的字符串。
    #[must_use]
    pub fn to_data_string(&self) -> String {
        self.0
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl std::fmt::Display for CV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_data_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cv_new() {
        let cv = CV::new(vec![0.5, 0.3, 0.8]);
        assert_eq!(cv.len(), 3);
        assert!(!cv.is_empty());
    }

    #[test]
    fn cv_get() {
        let cv = CV::new(vec![1.0, 2.0]);
        assert_eq!(cv.get(0), Some(1.0));
        assert_eq!(cv.get(1), Some(2.0));
        assert_eq!(cv.get(2), None);
    }

    #[test]
    fn cv_display() {
        let cv = CV::new(vec![0.5, 0.3]);
        let s = format!("{cv}");
        assert!(s.contains("0.5"));
        assert!(s.contains("0.3"));
    }
}
