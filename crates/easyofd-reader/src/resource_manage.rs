//! 资源管理器，按 ID 访问文档中出现的资源对象。
//!
//! 对应 Java: org.ofdrw.reader.ResourceManage
//!
//! Java 版通过解析 PublicRes.xml 和 DocumentRes.xml 加载字体、颜色空间、
//! 绘制参数、多媒体等资源到内存映射表。Rust 版复用 parser 已有的资源解析
//! 结果，提供按 ID 查找的能力。

use std::collections::HashMap;

/// 资源类型标识。
///
/// 对应 Java: `org.ofdrw.core.basicStructure.res.MediaType`（部分）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// 字体资源。
    Font,
    /// 图像资源。
    Image,
    /// 矢量图形资源。
    VectorGraphic,
    /// 颜色空间资源。
    ColorSpace,
    /// 绘制参数资源。
    DrawParam,
    /// 其他类型。
    Other,
}

/// 资源条目，描述文档中一个被引用的资源。
///
/// 对应 Java: `org.ofdrw.core.basicStructure.res.CT_MultiMedia`（简化版）
#[derive(Debug, Clone)]
pub struct ResourceItem {
    /// 资源 ID（文档内唯一）。
    pub id: String,
    /// 资源类型。
    pub resource_type: ResourceType,
    /// 资源文件在容器内的路径（相对于文档目录）。
    pub file_path: Option<String>,
    /// 资源文件格式（如 "PNG"、"JPEG"、"TTF"）。
    pub format: Option<String>,
}

impl ResourceItem {
    /// 创建新的资源条目。
    #[must_use]
    pub fn new(id: impl Into<String>, resource_type: ResourceType) -> Self {
        Self {
            id: id.into(),
            resource_type,
            file_path: None,
            format: None,
        }
    }

    /// 设置资源文件路径。
    #[must_use]
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// 设置资源格式。
    #[must_use]
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }
}

/// 资源管理器，维护文档中所有资源的 ID 索引。
///
/// 对应 Java: `org.ofdrw.reader.ResourceManage`
///
/// 提供按 ID 随机访问资源的能力。资源来源于 DocumentRes.xml 和
/// PublicRes.xml 中声明的资源序列。
#[derive(Debug, Clone, Default)]
pub struct ResourceManage {
    /// 所有资源的 ID 索引表。
    resources: HashMap<String, ResourceItem>,
    /// 按类型分组的 ID 集合（便于批量查询）。
    by_type: HashMap<ResourceType, Vec<String>>,
}

impl ResourceManage {
    /// 创建空的资源管理器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从资源条目列表构建资源管理器。
    #[must_use]
    pub fn from_items(items: Vec<ResourceItem>) -> Self {
        let mut mgr = Self::new();
        for item in items {
            mgr.insert(item);
        }
        mgr
    }

    /// 插入一个资源条目。如果 ID 已存在则覆盖。
    pub fn insert(&mut self, item: ResourceItem) {
        let id = item.id.clone();
        let resource_type = item.resource_type;
        self.by_type
            .entry(resource_type)
            .or_default()
            .push(id.clone());
        self.resources.insert(id, item);
    }

    /// 按 ID 获取资源条目。
    ///
    /// 对应 Java: `ResourceManage.get(String id)`
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ResourceItem> {
        self.resources.get(id)
    }

    /// 获取所有资源条目。
    #[must_use]
    pub fn all(&self) -> &HashMap<String, ResourceItem> {
        &self.resources
    }

    /// 按类型获取资源 ID 列表。
    #[must_use]
    pub fn ids_by_type(&self, resource_type: ResourceType) -> &[String] {
        self.by_type.get(&resource_type).map_or(&[], Vec::as_slice)
    }

    /// 获取指定类型的全部资源条目。
    #[must_use]
    pub fn items_by_type(&self, resource_type: ResourceType) -> Vec<&ResourceItem> {
        self.ids_by_type(resource_type)
            .iter()
            .filter_map(|id| self.resources.get(id))
            .collect()
    }

    /// 资源总数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// 资源管理器是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_manager() {
        let mgr = ResourceManage::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert!(mgr.get("nonexistent").is_none());
    }

    #[test]
    fn test_insert_and_get() {
        let mut mgr = ResourceManage::new();
        mgr.insert(
            ResourceItem::new("1", ResourceType::Image)
                .with_file_path("Res/image.png")
                .with_format("PNG"),
        );
        assert_eq!(mgr.len(), 1);
        let item = mgr.get("1").unwrap();
        assert_eq!(item.resource_type, ResourceType::Image);
        assert_eq!(item.file_path.as_deref(), Some("Res/image.png"));
    }

    #[test]
    fn test_from_items() {
        let items = vec![
            ResourceItem::new("1", ResourceType::Font),
            ResourceItem::new("2", ResourceType::Image),
            ResourceItem::new("3", ResourceType::Font),
        ];
        let mgr = ResourceManage::from_items(items);
        assert_eq!(mgr.len(), 3);
        assert_eq!(mgr.ids_by_type(ResourceType::Font).len(), 2);
        assert_eq!(mgr.ids_by_type(ResourceType::Image).len(), 1);
    }

    #[test]
    fn test_items_by_type() {
        let mut mgr = ResourceManage::new();
        mgr.insert(ResourceItem::new("10", ResourceType::ColorSpace));
        mgr.insert(ResourceItem::new("20", ResourceType::DrawParam));
        mgr.insert(ResourceItem::new("30", ResourceType::ColorSpace));
        let cs_items = mgr.items_by_type(ResourceType::ColorSpace);
        assert_eq!(cs_items.len(), 2);
    }

    #[test]
    fn test_insert_overwrite() {
        let mut mgr = ResourceManage::new();
        mgr.insert(ResourceItem::new("1", ResourceType::Image));
        mgr.insert(ResourceItem::new("1", ResourceType::Font));
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.get("1").unwrap().resource_type, ResourceType::Font);
    }
}
