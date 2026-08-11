//! 音频播放动作。
//!
//! 对应 Java: org.ofdrw.core.action.actionType.Sound

use super::OfdAction;

/// 音频播放动作。
///
/// 播放一个音频资源，对应 GB/T 33190 第 15 章的 Sound 动作。
///
/// 对应 Java: org.ofdrw.core.action.actionType.Sound
#[derive(Debug, Clone)]
pub struct Sound {
    /// 音频资源的引用 ID。
    ///
    /// 对应 Java: Sound.mediaRef (String)
    pub media_ref: String,

    /// 音量（0.0 ~ 1.0）。
    ///
    /// 对应 Java: Sound.volume (Double)
    pub volume: f64,

    /// 是否循环播放。
    ///
    /// 对应 Java: Sound.repeat (Boolean)
    pub repeat: bool,
}

impl Sound {
    /// 创建一个新的音频播放动作。
    ///
    /// 对应 Java: new Sound(String mediaRef)
    #[must_use]
    pub fn new(media_ref: impl Into<String>) -> Self {
        Self {
            media_ref: media_ref.into(),
            volume: 1.0,
            repeat: false,
        }
    }

    /// 设置音量。
    ///
    /// 对应 Java: Sound.setVolume(Double)
    #[must_use]
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// 设置是否循环播放。
    ///
    /// 对应 Java: Sound.setRepeat(Boolean)
    #[must_use]
    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }
}

impl OfdAction for Sound {
    fn to_xml_string(&self) -> String {
        format!(
            "<ofd:Sound MediaRef=\"{}\" Volume=\"{}\" Repeat=\"{}\"/>",
            self.media_ref, self.volume, self.repeat
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
    fn test_sound_new() {
        let sound = Sound::new("media_001");
        assert_eq!(sound.media_ref, "media_001");
        assert!((sound.volume - 1.0).abs() < f64::EPSILON);
        assert!(!sound.repeat);
    }

    #[test]
    fn test_sound_builder() {
        let sound = Sound::new("media_002").volume(0.5).repeat(true);
        assert_eq!(sound.media_ref, "media_002");
        assert!((sound.volume - 0.5).abs() < f64::EPSILON);
        assert!(sound.repeat);
    }

    #[test]
    fn test_sound_to_xml() {
        let sound = Sound::new("audio_1").volume(0.8).repeat(false);
        let xml = sound.to_xml_string();
        assert!(xml.contains("MediaRef=\"audio_1\""));
        assert!(xml.contains("Volume=\"0.8\""));
        assert!(xml.contains("Repeat=\"false\""));
        assert!(xml.contains("<ofd:Sound"));
    }

    #[test]
    fn test_sound_clone_debug() {
        let sound = Sound::new("m1");
        let sound2 = sound.clone();
        assert_eq!(sound2.media_ref, "m1");
        let dbg = format!("{sound:?}");
        assert!(dbg.contains("Sound"));
    }
}
