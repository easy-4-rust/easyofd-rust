//! 资源目录容器。
//!
//! 对应 Java: org.ofdrw.pkg.container.ResDir

use super::VirtualContainer;

/// 资源目录容器。
///
/// 对应 Java: org.ofdrw.pkg.container.ResDir
#[derive(Debug, Clone)]
pub struct ResDir {
    container: VirtualContainer,
}

impl ResDir {
    /// 创建资源目录。
    pub fn new() -> Self {
        Self {
            container: VirtualContainer::new("Res"),
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

impl Default for ResDir {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn res_dir_new() {
        let dir = ResDir::new();
        assert_eq!(dir.name(), "Res");
    }
}
