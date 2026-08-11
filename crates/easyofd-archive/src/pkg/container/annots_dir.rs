//! 注释目录容器。
//!
//! 对应 Java: org.ofdrw.pkg.container.AnnotsDir

use super::VirtualContainer;

/// 注释目录容器。
///
/// 对应 Java: org.ofdrw.pkg.container.AnnotsDir
#[derive(Debug, Clone)]
pub struct AnnotsDir {
    container: VirtualContainer,
}

impl AnnotsDir {
    /// 创建注释目录。
    pub fn new() -> Self {
        Self {
            container: VirtualContainer::new("Annots"),
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

impl Default for AnnotsDir {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annots_dir_new() {
        let dir = AnnotsDir::new();
        assert_eq!(dir.name(), "Annots");
    }
}
