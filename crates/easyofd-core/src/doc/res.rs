//! 资源文件。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res.Res
//! 资源是绘制图元时所需数据的集合

use crate::basic_type::ST_Loc;

/// 资源文件。
///
/// 资源是绘制图元时所需数据（如绘制参数、颜色空间、字形、图像、音视频等）的集合。
/// 对应 GB/T 33190-2016 第 7.9 节。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.Res
#[derive(Debug, Clone, Default)]
pub struct Res {
    /// 此资源文件的通用数据存储路径。
    pub base_loc: Option<ST_Loc>,
    /// 资源列表（XML 片段）。
    pub resources: Vec<String>,
}

impl Res {
    /// 创建新的资源文件。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 BaseLoc。
    #[must_use]
    pub fn base_loc(mut self, loc: ST_Loc) -> Self {
        self.base_loc = Some(loc);
        self
    }

    /// 添加资源。
    pub fn add_resource(&mut self, resource: impl Into<String>) {
        self.resources.push(resource.into());
    }

    /// 获取资源数量。
    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn res_new() {
        let r = Res::new();
        assert!(r.base_loc.is_none());
        assert_eq!(r.resource_count(), 0);
    }

    #[test]
    fn res_builder() {
        let r = Res::new().base_loc(ST_Loc::new("./Res"));
        assert!(r.base_loc.is_some());
    }

    #[test]
    fn res_add_resource() {
        let mut r = Res::new();
        r.add_resource("<ofd:Font/>");
        r.add_resource("<ofd:ColorSpace/>");
        assert_eq!(r.resource_count(), 2);
    }
}
