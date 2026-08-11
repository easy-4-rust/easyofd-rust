//! 整数标识符类型。
//!
//! 对应 Java: org.ofdrw.core.basicType.ST_ID

/// 标识，无符号整数，应在文档内唯一。0 标识无效标识符。
///
/// 示例：`1000`
///
/// 对应 Java: org.ofdrw.core.basicType.ST_ID
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ST_ID(u64);

impl ST_ID {
    /// 创建新的标识符，id 必须大于 0。
    pub fn new(id: u64) -> Result<Self, String> {
        if id == 0 {
            return Err("ID 必须大于 0".to_string());
        }
        Ok(Self(id))
    }

    /// 创建无效标识符（值为 0）。
    pub fn invalid() -> Self {
        Self(0)
    }

    /// 获取标识符值。
    pub fn get(&self) -> u64 {
        self.0
    }

    /// 创建引用标识符。
    pub fn as_ref_id(&self) -> super::ST_RefID {
        super::ST_RefID::new(self.0)
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        self.0.to_string()
    }

    /// 从字符串解析 ST_ID。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let id: u64 = s
            .trim()
            .parse()
            .map_err(|e| format!("解析 ST_ID 失败: {e}"))?;
        Ok(Self(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() {
        let id = ST_ID::new(1000).unwrap();
        assert_eq!(id.get(), 1000);
    }

    #[test]
    fn test_new_zero_rejected() {
        assert!(ST_ID::new(0).is_err());
    }

    #[test]
    fn test_to_xml_string() {
        let id = ST_ID::new(42).unwrap();
        assert_eq!(id.to_xml_string(), "42");
    }

    #[test]
    fn test_from_str() {
        let id = ST_ID::from_str("1000").unwrap();
        assert_eq!(id.get(), 1000);
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(ST_ID::from_str("abc").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let id = ST_ID::new(999).unwrap();
        let s = id.to_xml_string();
        let id2 = ST_ID::from_str(&s).unwrap();
        assert_eq!(id, id2);
    }
}
