//! 裁剪区域集合。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.clips.Clips

use super::clips::CT_Clip;

/// 裁剪区域集合，包含多个裁剪区域。
///
/// 对应 Java: org.ofdrw.core.pageDescription.clips.Clips
///
/// 在页面描述中，Clips 用于定义一组裁剪区域，
/// 应用到图元时只显示裁剪区域内的部分。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Default)]
pub struct Clips {
    /// 裁剪区域列表。
    pub clips: Vec<CT_Clip>,
}

impl Clips {
    /// 创建空裁剪区域集合。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加裁剪区域。
    pub fn add(&mut self, clip: CT_Clip) {
        self.clips.push(clip);
    }

    /// 获取裁剪区域数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.clips.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }

    /// 获取裁剪区域列表。
    #[must_use]
    pub fn items(&self) -> &[CT_Clip] {
        &self.clips
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clips_new() {
        let clips = Clips::new();
        assert!(clips.is_empty());
        assert_eq!(clips.len(), 0);
    }

    #[test]
    fn test_clips_add() {
        let mut clips = Clips::new();
        clips.add(CT_Clip::new());
        assert_eq!(clips.len(), 1);
        assert!(!clips.is_empty());
    }

    #[test]
    fn test_clips_items() {
        let mut clips = Clips::new();
        clips.add(CT_Clip::new());
        clips.add(CT_Clip::new());
        assert_eq!(clips.items().len(), 2);
    }
}
