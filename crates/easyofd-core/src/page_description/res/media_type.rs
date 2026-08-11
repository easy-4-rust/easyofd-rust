//! 多媒体资源类型枚举。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res.MediaType

/// 多媒体资源类型。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.MediaType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaType {
    /// 图像。
    Image,
    /// 音频。
    Audio,
    /// 视频。
    Video,
}

impl MediaType {
    /// 转为字符串。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::Audio => "Audio",
            Self::Video => "Video",
        }
    }

    /// 从字符串解析。
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "Image" => Some(Self::Image),
            "Audio" => Some(Self::Audio),
            "Video" => Some(Self::Video),
            _ => None,
        }
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_type_as_str() {
        assert_eq!(MediaType::Image.as_str(), "Image");
        assert_eq!(MediaType::Audio.as_str(), "Audio");
        assert_eq!(MediaType::Video.as_str(), "Video");
    }

    #[test]
    fn media_type_from_str() {
        assert_eq!(MediaType::from_str("Image"), Some(MediaType::Image));
        assert_eq!(MediaType::from_str("Audio"), Some(MediaType::Audio));
        assert_eq!(MediaType::from_str("Video"), Some(MediaType::Video));
        assert_eq!(MediaType::from_str("Unknown"), None);
    }

    #[test]
    fn media_type_display() {
        assert_eq!(MediaType::Image.to_string(), "Image");
    }
}
