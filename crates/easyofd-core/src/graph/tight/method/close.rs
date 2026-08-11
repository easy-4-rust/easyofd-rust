//! 闭合路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Close

/// 闭合路径方法。
///
/// 自动闭合到当前路径的起始点，并以该点为当前点。
///
/// 对应 Java: org.ofdrw.core.graph.tight.method.Close
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Close;

impl Close {
    /// 创建闭合命令。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 序列化为缩写数据字符串（C 命令格式）。
    #[must_use]
    pub fn to_abbreviated_string(&self) -> &'static str {
        "C"
    }
}

impl Default for Close {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Close {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("C")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_new() {
        let c = Close::new();
        assert_eq!(c.to_abbreviated_string(), "C");
    }

    #[test]
    fn close_display() {
        assert_eq!(format!("{Close}"), "C");
    }

    #[test]
    fn close_default() {
        let c = Close;
        assert_eq!(c.to_abbreviated_string(), "C");
    }
}
