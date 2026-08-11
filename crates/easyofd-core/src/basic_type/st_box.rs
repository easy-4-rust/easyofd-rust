//! 矩形框类型。
//!
//! 对应 Java: org.ofdrw.core.basicType.ST_Box

/// 矩形区域，以空格分割，前两个值代表了该矩形的左上角的坐标，
/// 后两个值依次表示该矩形的宽和高，可以是整数或者浮点数，后两个值应大于0。
///
/// 示例：`10 10 50 50`
///
/// 对应 Java: org.ofdrw.core.basicType.ST_Box
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct ST_Box {
    /// 左上角 X 坐标
    pub top_left_x: f64,
    /// 左上角 Y 坐标
    pub top_left_y: f64,
    /// 宽度
    pub width: f64,
    /// 高度
    pub height: f64,
}

impl ST_Box {
    /// 创建新的矩形框。
    pub fn new(top_left_x: f64, top_left_y: f64, width: f64, height: f64) -> Self {
        Self {
            top_left_x,
            top_left_y,
            width,
            height,
        }
    }

    /// 获取左上角坐标点。
    pub fn top_left_pos(&self) -> (f64, f64) {
        (self.top_left_x, self.top_left_y)
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        format!(
            "{} {} {} {}",
            self.top_left_x, self.top_left_y, self.width, self.height
        )
    }

    /// 从字符串解析 ST_Box。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 4 {
            return Err(format!("ST_Box 需要 4 个值，得到 {} 个", parts.len()));
        }
        let top_left_x: f64 = parts[0]
            .parse()
            .map_err(|e| format!("解析 top_left_x 失败: {e}"))?;
        let top_left_y: f64 = parts[1]
            .parse()
            .map_err(|e| format!("解析 top_left_y 失败: {e}"))?;
        let width: f64 = parts[2]
            .parse()
            .map_err(|e| format!("解析 width 失败: {e}"))?;
        let height: f64 = parts[3]
            .parse()
            .map_err(|e| format!("解析 height 失败: {e}"))?;
        Ok(Self {
            top_left_x,
            top_left_y,
            width,
            height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_xml_string() {
        let b = ST_Box::new(10.0, 20.0, 50.0, 80.0);
        assert_eq!(b.to_xml_string(), "10 20 50 80");
    }

    #[test]
    fn test_from_str() {
        let b = ST_Box::from_str("10 20 50 80").unwrap();
        assert_eq!(b.top_left_x, 10.0);
        assert_eq!(b.top_left_y, 20.0);
        assert_eq!(b.width, 50.0);
        assert_eq!(b.height, 80.0);
    }

    #[test]
    fn test_from_str_floats() {
        let b = ST_Box::from_str("1.5 2.5 10.0 20.0").unwrap();
        assert_eq!(b.top_left_x, 1.5);
        assert_eq!(b.top_left_y, 2.5);
    }

    #[test]
    fn test_from_str_invalid_count() {
        assert!(ST_Box::from_str("1 2 3").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let b = ST_Box::new(100.0, 200.0, 300.0, 400.0);
        let s = b.to_xml_string();
        let b2 = ST_Box::from_str(&s).unwrap();
        assert_eq!(b, b2);
    }
}
