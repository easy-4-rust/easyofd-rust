//! 模板目录容器。
//!
//! 对应 Java: org.ofdrw.pkg.container.TempsDir

use super::VirtualContainer;

/// 模板目录容器。
///
/// 对应 Java: org.ofdrw.pkg.container.TempsDir
#[derive(Debug, Clone)]
pub struct TempsDir {
    container: VirtualContainer,
}

impl TempsDir {
    /// 创建模板目录。
    pub fn new() -> Self {
        Self {
            container: VirtualContainer::new("Temps"),
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

impl Default for TempsDir {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temps_dir_new() {
        let dir = TempsDir::new();
        assert_eq!(dir.name(), "Temps");
    }
}
