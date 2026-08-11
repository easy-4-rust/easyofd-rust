//! 页面目录容器。
//!
//! 对应 Java: org.ofdrw.pkg.container.PageDir

use super::VirtualContainer;

/// 页面目录容器。
///
/// 对应 Java: org.ofdrw.pkg.container.PageDir
#[derive(Debug, Clone)]
pub struct PageDir {
    container: VirtualContainer,
}

impl PageDir {
    /// 创建页面目录。
    pub fn new(page_index: u32) -> Self {
        Self {
            container: VirtualContainer::new(format!("Page_{page_index}")),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_dir_new() {
        let dir = PageDir::new(0);
        assert_eq!(dir.name(), "Page_0");
    }

    #[test]
    fn page_dir_new_index() {
        let dir = PageDir::new(5);
        assert_eq!(dir.name(), "Page_5");
    }
}
