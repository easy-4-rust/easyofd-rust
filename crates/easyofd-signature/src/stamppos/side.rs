//! 骑缝章所在边枚举。
//!
//! 对应 Java: `org.ofdrw.sign.stamppos.Side`

/// 骑缝章所在的边。
///
/// 对应 Java: `org.ofdrw.sign.stamppos.Side`
///
/// 指定骑缝章贴在页面的哪一侧。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// 左侧骑缝。
    Left,
    /// 右侧骑缝。
    Right,
    /// 顶部骑缝。
    Top,
    /// 底部骑缝。
    Bottom,
}

impl Side {
    /// 获取所有方向。
    #[must_use]
    pub fn all() -> &'static [Side] {
        &[Side::Left, Side::Right, Side::Top, Side::Bottom]
    }

    /// 是否为水平方向（Left/Right）。
    #[must_use]
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// 是否为垂直方向（Top/Bottom）。
    #[must_use]
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
            Self::Top => write!(f, "Top"),
            Self::Bottom => write!(f, "Bottom"),
        }
    }
}

impl std::str::FromStr for Side {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Left" | "left" | "LEFT" => Ok(Self::Left),
            "Right" | "right" | "RIGHT" => Ok(Self::Right),
            "Top" | "top" | "TOP" => Ok(Self::Top),
            "Bottom" | "bottom" | "BOTTOM" => Ok(Self::Bottom),
            _ => Err(format!("未知的骑缝章方向: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_and_parse() {
        for side in Side::all() {
            let s = side.to_string();
            let parsed: Side = s.parse().unwrap();
            assert_eq!(parsed, *side);
        }
    }

    #[test]
    fn is_horizontal() {
        assert!(Side::Left.is_horizontal());
        assert!(Side::Right.is_horizontal());
        assert!(!Side::Top.is_horizontal());
        assert!(!Side::Bottom.is_horizontal());
    }

    #[test]
    fn is_vertical() {
        assert!(Side::Top.is_vertical());
        assert!(Side::Bottom.is_vertical());
        assert!(!Side::Left.is_vertical());
        assert!(!Side::Right.is_vertical());
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!("left".parse::<Side>().unwrap(), Side::Left);
        assert_eq!("RIGHT".parse::<Side>().unwrap(), Side::Right);
        assert!("unknown".parse::<Side>().is_err());
    }

    #[test]
    fn all_contains_four() {
        assert_eq!(Side::all().len(), 4);
    }
}
