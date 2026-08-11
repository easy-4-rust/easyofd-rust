//! OFD 包根目录容器。
//!
//! 对应 Java: org.ofdrw.pkg.container.OFDDir

use super::VirtualContainer;

/// OFD 包根目录容器。
///
/// 对应 Java: org.ofdrw.pkg.container.OFDDir
#[derive(Debug, Clone)]
pub struct OfdPkgDir {
    /// 根容器。
    container: VirtualContainer,
}

impl OfdPkgDir {
    /// 创建 OFD 包根目录。
    pub fn new() -> Self {
        Self {
            container: VirtualContainer::new("OFD"),
        }
    }

    /// 获取根容器。
    #[must_use]
    pub fn container(&self) -> &VirtualContainer {
        &self.container
    }

    /// 获取根容器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        self.container.name()
    }
}

impl Default for OfdPkgDir {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ofd_pkg_dir_new() {
        let dir = OfdPkgDir::new();
        assert_eq!(dir.name(), "OFD");
    }

    #[test]
    fn ofd_pkg_dir_default() {
        let dir = OfdPkgDir::default();
        assert_eq!(dir.name(), "OFD");
    }
}
