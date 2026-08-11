//! 目标类型枚举。
//!
//! 对应 Java: org.ofdrw.core.action.DestType

/// 目标显示类型。
///
/// 定义页面跳转目标的显示方式，对应 GB/T 33190 第 15 章。
///
/// 对应 Java: org.ofdrw.core.action.DestType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DestType {
    /// 适合页面。
    ///
    /// 将页面缩放到完全适合窗口。
    Fit,

    /// 适合水平方向。
    ///
    /// 将页面水平方向适合窗口，垂直方向以指定 Y 坐标定位。
    FitH,

    /// 适合垂直方向。
    ///
    /// 将页面垂直方向适合窗口，水平方向以指定 X 坐标定位。
    FitV,

    /// 适合矩形区域。
    ///
    /// 将页面缩放到适合指定的矩形区域。
    FitB,

    /// 适合水平方向（含边界）。
    ///
    /// 类似 FitH，但包含边界区域。
    FitBH,

    /// 适合垂直方向（含边界）。
    ///
    /// 类似 FitV，但包含边界区域。
    FitBV,

    /// 指定缩放级别和位置。
    ///
    /// 以指定的缩放级别显示页面，左上角位于 (X, Y)。
    XYZ,
}

impl DestType {
    /// 返回目标类型的 OFD XML 属性值。
    ///
    /// 对应 Java: DestType 的 toString()/value
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fit => "Fit",
            Self::FitH => "FitH",
            Self::FitV => "FitV",
            Self::FitB => "FitB",
            Self::FitBH => "FitBH",
            Self::FitBV => "FitBV",
            Self::XYZ => "XYZ",
        }
    }

    /// 从字符串解析目标类型。
    ///
    /// # Errors
    ///
    /// 如果字符串不匹配任何已知目标类型，返回错误。
    pub fn from_str_value(s: &str) -> Result<Self, String> {
        match s {
            "Fit" => Ok(Self::Fit),
            "FitH" => Ok(Self::FitH),
            "FitV" => Ok(Self::FitV),
            "FitB" => Ok(Self::FitB),
            "FitBH" => Ok(Self::FitBH),
            "FitBV" => Ok(Self::FitBV),
            "XYZ" => Ok(Self::XYZ),
            _ => Err(format!("unknown DestType: {s}")),
        }
    }
}

impl std::fmt::Display for DestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dest_type_as_str() {
        assert_eq!(DestType::Fit.as_str(), "Fit");
        assert_eq!(DestType::FitH.as_str(), "FitH");
        assert_eq!(DestType::FitV.as_str(), "FitV");
        assert_eq!(DestType::FitB.as_str(), "FitB");
        assert_eq!(DestType::FitBH.as_str(), "FitBH");
        assert_eq!(DestType::FitBV.as_str(), "FitBV");
        assert_eq!(DestType::XYZ.as_str(), "XYZ");
    }

    #[test]
    fn test_dest_type_from_str_roundtrip() {
        let variants = [
            DestType::Fit,
            DestType::FitH,
            DestType::FitV,
            DestType::FitB,
            DestType::FitBH,
            DestType::FitBV,
            DestType::XYZ,
        ];
        for v in &variants {
            let s = v.as_str();
            let parsed = DestType::from_str_value(s).unwrap();
            assert_eq!(*v, parsed);
        }
    }

    #[test]
    fn test_dest_type_from_str_invalid() {
        assert!(DestType::from_str_value("INVALID").is_err());
    }

    #[test]
    fn test_dest_type_display() {
        assert_eq!(format!("{}", DestType::XYZ), "XYZ");
    }

    #[test]
    fn test_dest_type_clone_copy() {
        let a = DestType::FitH;
        let b = a;
        assert_eq!(a, b);
        let c = a;
        assert_eq!(a, c);
    }
}
