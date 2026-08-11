//! 页面集合目录容器。
//!
//! 对应 Java: org.ofdrw.pkg.container.PagesDir

use super::VirtualContainer;

/// 页面集合目录容器。
///
/// 对应 Java: org.ofdrw.pkg.container.PagesDir
#[derive(Debug, Clone)]
pub struct PagesDir {
    container: VirtualContainer,
}

impl PagesDir {
    /// 创建页面集合目录。
    pub fn new() -> Self {
        Self {
            container: VirtualContainer::new("Pages"),
        }
    }

    /// 获取容器。
    #[must_use]
    pub fn container(&self) -> &VirtualContainer {
        &self.container
    }

    /// 获取目录名。
    #[must_use]
    pub fn name(&self) -> &str {
        self.container.name()
    }
}

impl Default for PagesDir {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_dir_new() {
        let dir = PagesDir::new();
        assert_eq!(dir.name(), "Pages");
    }
}
