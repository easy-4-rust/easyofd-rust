//! 绘制参数缓存。
//!
//! 对应 Java 版 `ofdrw-graphics2d` 中的绘制参数管理，
//! 通过 ID 缓存 [`CT_DrawParam`] 实例，避免重复创建相同的绘制参数。

use std::collections::HashMap;

use easyofd_core::page_description::draw_param::CT_DrawParam;

/// 绘制参数缓存。
///
/// 以 `u64` ID 为键缓存 [`CT_DrawParam`] 实例，支持：
/// - 插入 / 查询 / 移除
/// - 基于线宽查找或自动创建
/// - 批量遍历
#[derive(Debug, Clone)]
pub struct DrawParamCache {
    /// 缓存存储，ID → 绘制参数。
    cache: HashMap<u64, CT_DrawParam>,
    /// 下一个自动分配的 ID。
    next_id: u64,
}

impl DrawParamCache {
    /// 创建空缓存。
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            next_id: 1,
        }
    }

    /// 以指定 ID 插入绘制参数。
    ///
    /// 如果 ID 已存在，覆盖旧值。
    pub fn insert(&mut self, id: u64, param: CT_DrawParam) {
        self.cache.insert(id, param);
    }

    /// 自动分配 ID 并插入绘制参数，返回分配的 ID。
    pub fn insert_auto(&mut self, param: CT_DrawParam) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.cache.insert(id, param);
        id
    }

    /// 根据 ID 获取绘制参数的引用。
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&CT_DrawParam> {
        self.cache.get(&id)
    }

    /// 根据 ID 获取绘制参数的可变引用。
    pub fn get_mut(&mut self, id: u64) -> Option<&mut CT_DrawParam> {
        self.cache.get_mut(&id)
    }

    /// 移除指定 ID 的绘制参数，返回被移除的参数。
    pub fn remove(&mut self, id: u64) -> Option<CT_DrawParam> {
        self.cache.remove(&id)
    }

    /// 查找或创建指定线宽的绘制参数。
    ///
    /// 如果缓存中已有相同线宽的参数，返回其 ID；
    /// 否则创建新参数并插入缓存。
    pub fn get_or_insert_with_line_width(&mut self, line_width: f64) -> u64 {
        // 先查找已有相同线宽的参数
        for (&id, param) in &self.cache {
            if param.line_width() == Some(line_width) {
                return id;
            }
        }
        // 未找到，创建新参数
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(line_width);
        self.insert_auto(dp)
    }

    /// 返回缓存中的条目数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 判断缓存是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// 遍历缓存中所有条目。
    pub fn for_each(&self, mut f: impl FnMut(u64, &CT_DrawParam)) {
        for (&id, param) in &self.cache {
            f(id, param);
        }
    }

    /// 清空缓存。
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for DrawParamCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache_is_empty() {
        let cache = DrawParamCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = DrawParamCache::new();
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(1.5);
        cache.insert(10, dp.clone());
        let got = cache.get(10).expect("应存在");
        assert_eq!(got.line_width(), Some(1.5));
    }

    #[test]
    fn test_insert_auto() {
        let mut cache = DrawParamCache::new();
        let mut dp1 = CT_DrawParam::new();
        dp1.set_line_width(1.0);
        let id1 = cache.insert_auto(dp1);
        assert_eq!(id1, 1);

        let mut dp2 = CT_DrawParam::new();
        dp2.set_line_width(2.0);
        let id2 = cache.insert_auto(dp2);
        assert_eq!(id2, 2);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_get_or_insert_with_line_width() {
        let mut cache = DrawParamCache::new();
        let id1 = cache.get_or_insert_with_line_width(0.5);
        let id2 = cache.get_or_insert_with_line_width(0.5);
        // 相同线宽应返回同一 ID
        assert_eq!(id1, id2);

        let id3 = cache.get_or_insert_with_line_width(1.0);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_remove() {
        let mut cache = DrawParamCache::new();
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(3.0);
        cache.insert(5, dp);
        assert_eq!(cache.len(), 1);
        let removed = cache.remove(5);
        assert!(removed.is_some());
        assert!(cache.is_empty());
    }

    #[test]
    fn test_get_mut() {
        let mut cache = DrawParamCache::new();
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(1.0);
        cache.insert(1, dp);
        if let Some(param) = cache.get_mut(1) {
            param.set_line_width(2.0);
        }
        assert_eq!(cache.get(1).unwrap().line_width(), Some(2.0));
    }

    #[test]
    fn test_for_each() {
        let mut cache = DrawParamCache::new();
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(1.0);
        cache.insert(1, dp);
        let mut count = 0;
        cache.for_each(|_id, _param| {
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn test_clear() {
        let mut cache = DrawParamCache::new();
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(1.0);
        cache.insert(1, dp);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_default() {
        let cache = DrawParamCache::default();
        assert!(cache.is_empty());
    }
}
