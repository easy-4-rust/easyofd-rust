//! 视频播放类型枚举。
//!
//! 对应 Java: org.ofdrw.core.action.PlayType

/// 视频播放类型。
///
/// 定义视频/音频的播放方式，对应 GB/T 33190 第 15 章。
///
/// 对应 Java: org.ofdrw.core.action.PlayType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayType {
    /// 播放。
    ///
    /// 开始播放媒体资源。
    Play,

    /// 暂停。
    ///
    /// 暂停当前播放。
    Pause,

    /// 停止。
    ///
    /// 停止当前播放并重置到起始位置。
    Stop,

    /// 恢复。
    ///
    /// 恢复暂停的播放。
    Resume,
}

impl PlayType {
    /// 返回播放类型的 OFD XML 属性值。
    ///
    /// 对应 Java: PlayType 的 toString()/value
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::Pause => "Pause",
            Self::Stop => "Stop",
            Self::Resume => "Resume",
        }
    }

    /// 从字符串解析播放类型。
    ///
    /// # Errors
    ///
    /// 如果字符串不匹配任何已知播放类型，返回错误。
    pub fn from_str_value(s: &str) -> Result<Self, String> {
        match s {
            "Play" => Ok(Self::Play),
            "Pause" => Ok(Self::Pause),
            "Stop" => Ok(Self::Stop),
            "Resume" => Ok(Self::Resume),
            _ => Err(format!("unknown PlayType: {s}")),
        }
    }
}

impl std::fmt::Display for PlayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_type_as_str() {
        assert_eq!(PlayType::Play.as_str(), "Play");
        assert_eq!(PlayType::Pause.as_str(), "Pause");
        assert_eq!(PlayType::Stop.as_str(), "Stop");
        assert_eq!(PlayType::Resume.as_str(), "Resume");
    }

    #[test]
    fn test_play_type_from_str_roundtrip() {
        let variants = [
            PlayType::Play,
            PlayType::Pause,
            PlayType::Stop,
            PlayType::Resume,
        ];
        for v in &variants {
            let s = v.as_str();
            let parsed = PlayType::from_str_value(s).unwrap();
            assert_eq!(*v, parsed);
        }
    }

    #[test]
    fn test_play_type_from_str_invalid() {
        assert!(PlayType::from_str_value("INVALID").is_err());
    }

    #[test]
    fn test_play_type_display() {
        assert_eq!(format!("{}", PlayType::Play), "Play");
    }

    #[test]
    fn test_play_type_clone_copy() {
        let a = PlayType::Stop;
        let b = a;
        assert_eq!(a, b);
        let c = a;
        assert_eq!(a, c);
    }
}
