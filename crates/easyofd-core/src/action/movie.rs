//! 视频播放动作。
//!
//! 对应 Java: org.ofdrw.core.action.actionType.Movie

use super::{OfdAction, PlayType};

/// 视频播放动作。
///
/// 控制视频资源的播放，对应 GB/T 33190 第 15 章的 Movie 动作。
///
/// 对应 Java: org.ofdrw.core.action.actionType.Movie
#[derive(Debug, Clone)]
pub struct Movie {
    /// 视频资源的引用 ID。
    ///
    /// 对应 Java: Movie.mediaRef (String)
    pub media_ref: String,

    /// 播放类型。
    ///
    /// 对应 Java: Movie.type (PlayType)
    pub play_type: PlayType,
}

impl Movie {
    /// 创建一个新的视频播放动作。
    ///
    /// 对应 Java: new Movie(String mediaRef, PlayType type)
    #[must_use]
    pub fn new(media_ref: impl Into<String>, play_type: PlayType) -> Self {
        Self {
            media_ref: media_ref.into(),
            play_type,
        }
    }
}

impl OfdAction for Movie {
    fn to_xml_string(&self) -> String {
        format!(
            "<ofd:Movie MediaRef=\"{}\" Type=\"{}\"/>",
            self.media_ref, self.play_type
        )
    }

    fn clone_box(&self) -> Box<dyn OfdAction> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movie_new() {
        let movie = Movie::new("video_001", PlayType::Play);
        assert_eq!(movie.media_ref, "video_001");
        assert_eq!(movie.play_type, PlayType::Play);
    }

    #[test]
    fn test_movie_to_xml_play() {
        let movie = Movie::new("vid_1", PlayType::Play);
        let xml = movie.to_xml_string();
        assert!(xml.contains("MediaRef=\"vid_1\""));
        assert!(xml.contains("Type=\"Play\""));
        assert!(xml.contains("<ofd:Movie"));
        assert!(xml.ends_with("/>"));
    }

    #[test]
    fn test_movie_to_xml_stop() {
        let movie = Movie::new("vid_2", PlayType::Stop);
        let xml = movie.to_xml_string();
        assert!(xml.contains("Type=\"Stop\""));
    }

    #[test]
    fn test_movie_clone_debug() {
        let movie = Movie::new("m1", PlayType::Pause);
        let movie2 = movie.clone();
        assert_eq!(movie2.media_ref, "m1");
        assert_eq!(movie2.play_type, PlayType::Pause);
        let dbg = format!("{movie:?}");
        assert!(dbg.contains("Movie"));
    }
}
