//! 引用标识符类型。
//!
//! 对应 Java: org.ofdrw.core.basicType.ST_RefID

/// 标识符引用，无符号整数，此标识符应为文档内已定义的标识符。
///
/// 示例：`1000`
///
/// 对应 Java: org.ofdrw.core.basicType.ST_RefID
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ST_RefID(u64);

impl ST_RefID {
    /// 创建新的引用标识符。
    pub fn new(ref_id: u64) -> Self {
        Self(ref_id)
    }

    /// 获取引用标识符值。
    pub fn get(&self) -> u64 {
        self.0
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        self.0.to_string()
    }

    /// 从字符串解析 ST_RefID。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let id: u64 = s
            .trim()
            .parse()
            .map_err(|e| format!("解析 ST_RefID 失败: {e}"))?;
        Ok(Self(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = ST_RefID::new(1000);
        assert_eq!(r.get(), 1000);
    }

    #[test]
    fn test_to_xml_string() {
        let r = ST_RefID::new(42);
        assert_eq!(r.to_xml_string(), "42");
    }

    #[test]
    fn test_from_str() {
        let r = ST_RefID::from_str("1000").unwrap();
        assert_eq!(r.get(), 1000);
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(ST_RefID::from_str("abc").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let r = ST_RefID::new(999);
        let s = r.to_xml_string();
        let r2 = ST_RefID::from_str(&s).unwrap();
        assert_eq!(r, r2);
    }
}
