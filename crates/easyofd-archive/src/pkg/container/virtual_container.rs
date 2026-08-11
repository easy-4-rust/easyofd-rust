//! 虚拟容器基类。
//!
//! 对应 Java: org.ofdrw.pkg.container.VirtualContainer

/// 虚拟容器。
///
/// OFD 包中的目录或文件容器，提供路径管理能力。
///
/// 对应 Java: org.ofdrw.pkg.container.VirtualContainer
#[derive(Debug, Clone)]
pub struct VirtualContainer {
    /// 容器名称（目录名或文件名）。
    name: String,
    /// 子容器列表。
    children: Vec<VirtualContainer>,
}

impl VirtualContainer {
    /// 创建新的虚拟容器。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
        }
    }

    /// 获取容器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 添加子容器。
    pub fn add_child(&mut self, child: VirtualContainer) {
        self.children.push(child);
    }

    /// 获取子容器数量。
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// 获取子容器列表。
    #[must_use]
    pub fn children(&self) -> &[VirtualContainer] {
        &self.children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_container_new() {
        let vc = VirtualContainer::new("Doc_0");
        assert_eq!(vc.name(), "Doc_0");
        assert_eq!(vc.child_count(), 0);
    }

    #[test]
    fn virtual_container_add_child() {
        let mut vc = VirtualContainer::new("root");
        vc.add_child(VirtualContainer::new("child1"));
        vc.add_child(VirtualContainer::new("child2"));
        assert_eq!(vc.child_count(), 2);
        assert_eq!(vc.children()[0].name(), "child1");
    }

    #[test]
    fn virtual_container_clone() {
        let vc = VirtualContainer::new("test");
        let vc2 = vc.clone();
        assert_eq!(vc2.name(), "test");
    }
}
