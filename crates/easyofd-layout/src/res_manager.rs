//! 资源管理器——字体、图片、绘制参数的注册与去重。
//!
//! 对应 Java: org.ofdrw.layout.engine.ResManager

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use easyofd_core::CT_Font;
use easyofd_core::page_description::{CT_DrawParam, CT_MultiMedia, MediaType};

use crate::exist_ct_font::ExistCtFont;

/// 资源管理器。
///
/// 对应 Java: org.ofdrw.layout.engine.ResManager
///
/// 管理待加入文档中的所有资源（字体、图片、绘制参数），自动分配唯一 ID，
/// 并通过内容哈希去重：相同资源只注册一次，后续调用返回已有 ID。
///
/// 线程安全：ID 分配基于 `AtomicU32`，内部注册表使用 `RefCell`（单线程场景）。
#[derive(Debug)]
pub struct ResManager {
    /// 最大对象 ID 计数器。
    max_unit_id: AtomicU32,
    /// 字体注册表：key = 小写字体名 → (id, CT_Font)。
    fonts: std::cell::RefCell<HashMap<String, (u32, CT_Font)>>,
    /// 图片注册表：key = 文件路径 → (id, CT_MultiMedia)。
    images: std::cell::RefCell<HashMap<String, (u32, CT_MultiMedia)>>,
    /// 绘制参数注册表：key = XML 内容哈希 → (id, CT_DrawParam)。
    draw_params: std::cell::RefCell<HashMap<u64, (u32, CT_DrawParam)>>,
}

impl ResManager {
    /// 创建资源管理器，ID 从 `initial_max_id` 之后开始分配。
    #[must_use]
    pub fn new(initial_max_id: u32) -> Self {
        Self {
            max_unit_id: AtomicU32::new(initial_max_id),
            fonts: std::cell::RefCell::new(HashMap::new()),
            images: std::cell::RefCell::new(HashMap::new()),
            draw_params: std::cell::RefCell::new(HashMap::new()),
        }
    }

    /// 分配并返回下一个唯一 ID。
    pub fn alloc_id(&self) -> u32 {
        self.max_unit_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// 获取当前最大对象 ID。
    #[must_use]
    pub fn max_unit_id(&self) -> u32 {
        self.max_unit_id.load(Ordering::Relaxed)
    }

    /// 注册字体资源，返回其对象 ID。
    ///
    /// 如果同名字体已注册，直接返回已有 ID（去重）。
    pub fn add_font(&self, font: &CT_Font) -> u32 {
        let key = font.font_name.to_lowercase();
        let mut fonts = self.fonts.borrow_mut();
        if let Some(&(existing_id, _)) = fonts.get(&key) {
            return existing_id;
        }
        let id = self.alloc_id();
        let mut registered = font.clone();
        registered.id = id;
        fonts.insert(key, (id, registered));
        id
    }

    /// 按字体名称查找已注册字体。
    #[must_use]
    pub fn get_font(&self, name: &str) -> Option<ExistCtFont> {
        let key = name.to_lowercase();
        let fonts = self.fonts.borrow();
        fonts.get(&key).map(|&(id, ref ct)| {
            ExistCtFont::new(id, &ct.font_name)
                .family_name(ct.family_name.as_deref().unwrap_or(""))
                .embedded(ct.font_file.is_some())
        })
    }

    /// 注册图片资源，返回其对象 ID。
    ///
    /// 如果同路径图片已注册，直接返回已有 ID（去重）。
    pub fn add_image(&self, media: &CT_MultiMedia) -> u32 {
        let key = media.file.clone().unwrap_or_default();
        let mut images = self.images.borrow_mut();
        if let Some(&(existing_id, _)) = images.get(&key) {
            return existing_id;
        }
        let id = self.alloc_id();
        let mut registered = media.clone();
        registered.id = id;
        images.insert(key, (id, registered));
        id
    }

    /// 便捷方法：按路径和格式注册图片资源。
    pub fn add_image_by_path(&self, file_path: &str, format: &str) -> u32 {
        let media = CT_MultiMedia::new(0, MediaType::Image)
            .format(format)
            .file(file_path);
        self.add_image(&media)
    }

    /// 注册绘制参数，返回其对象 ID。
    ///
    /// 通过 XML 序列化去重：相同参数内容只注册一次。
    pub fn add_draw_param(&self, param: &CT_DrawParam) -> u32 {
        let xml_key = {
            // 使用固定 sentinel ID 序列化，确保去重不受实际 ID 影响
            let mut clean = param.clone();
            clean.set_id(easyofd_core::ST_ID::new(1).expect("常量 ID 1 有效"));
            hash_string(&clean.to_xml_string())
        };
        let mut params = self.draw_params.borrow_mut();
        if let Some(&(existing_id, _)) = params.get(&xml_key) {
            return existing_id;
        }
        let id = self.alloc_id();
        let mut registered = param.clone();
        registered.set_id(easyofd_core::ST_ID::new(u64::from(id)).expect("u32 转 u64 生成有效 ID"));
        params.insert(xml_key, (id, registered));
        id
    }

    /// 获取已注册的字体列表。
    #[must_use]
    pub fn font_count(&self) -> usize {
        self.fonts.borrow().len()
    }

    /// 获取已注册的图片数量。
    #[must_use]
    pub fn image_count(&self) -> usize {
        self.images.borrow().len()
    }

    /// 获取已注册的绘制参数数量。
    #[must_use]
    pub fn draw_param_count(&self) -> usize {
        self.draw_params.borrow().len()
    }
}

/// 简单字符串哈希（FNV-1a 64-bit），用于去重 key。
fn hash_string(s: &str) -> u64 {
    let mut hash: u64 = 0x_CBF2_9CE4_8422_2325;
    for byte in s.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01B3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 基本创建 ---

    #[test]
    fn new_starts_at_given_id() {
        let rm = ResManager::new(10);
        assert_eq!(rm.max_unit_id(), 10);
    }

    #[test]
    fn alloc_id_increments() {
        let rm = ResManager::new(0);
        assert_eq!(rm.alloc_id(), 1);
        assert_eq!(rm.alloc_id(), 2);
        assert_eq!(rm.alloc_id(), 3);
        assert_eq!(rm.max_unit_id(), 3);
    }

    // --- 字体去重 ---

    #[test]
    fn add_font_dedup() {
        let rm = ResManager::new(0);
        let f1 = CT_Font::new(0, "SimSun");
        let f2 = CT_Font::new(0, "SimSun"); // 同名
        let f3 = CT_Font::new(0, "SimHei"); // 不同名

        let id1 = rm.add_font(&f1);
        let id2 = rm.add_font(&f2);
        let id3 = rm.add_font(&f3);

        assert_eq!(id1, id2, "同名字体应返回相同 ID");
        assert_ne!(id1, id3, "不同名字体应返回不同 ID");
        assert_eq!(rm.font_count(), 2);
    }

    #[test]
    fn add_font_case_insensitive() {
        let rm = ResManager::new(0);
        let id1 = rm.add_font(&CT_Font::new(0, "SimSun"));
        let id2 = rm.add_font(&CT_Font::new(0, "simsun"));
        assert_eq!(id1, id2);
    }

    #[test]
    fn get_font_returns_registered() {
        let rm = ResManager::new(0);
        rm.add_font(&CT_Font::new(0, "SimSun").family_name("宋体"));
        let found = rm.get_font("SimSun").expect("字体应存在");
        assert_eq!(found.font_name, "SimSun");
        assert_eq!(found.family_name.as_deref(), Some("宋体"));
    }

    #[test]
    fn get_font_not_found() {
        let rm = ResManager::new(0);
        assert!(rm.get_font("NotExist").is_none());
    }

    // --- 图片去重 ---

    #[test]
    fn add_image_dedup() {
        let rm = ResManager::new(0);
        let m1 = CT_MultiMedia::new(0, MediaType::Image).file("a.png");
        let m2 = CT_MultiMedia::new(0, MediaType::Image).file("a.png");
        let m3 = CT_MultiMedia::new(0, MediaType::Image).file("b.png");

        let id1 = rm.add_image(&m1);
        let id2 = rm.add_image(&m2);
        let id3 = rm.add_image(&m3);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(rm.image_count(), 2);
    }

    #[test]
    fn add_image_by_path() {
        let rm = ResManager::new(0);
        let id = rm.add_image_by_path("test.png", "PNG");
        assert_eq!(id, 1);
        assert_eq!(rm.image_count(), 1);
    }

    // --- 绘制参数去重 ---

    #[test]
    fn add_draw_param_dedup() {
        let rm = ResManager::new(0);
        let mut dp1 = CT_DrawParam::new();
        dp1.set_line_width(2.0);
        let mut dp2 = CT_DrawParam::new();
        dp2.set_line_width(2.0); // 相同参数
        let mut dp3 = CT_DrawParam::new();
        dp3.set_line_width(5.0); // 不同参数

        let id1 = rm.add_draw_param(&dp1);
        let id2 = rm.add_draw_param(&dp2);
        let id3 = rm.add_draw_param(&dp3);

        assert_eq!(id1, id2, "相同参数应返回相同 ID");
        assert_ne!(id1, id3, "不同参数应返回不同 ID");
        assert_eq!(rm.draw_param_count(), 2);
    }

    // --- 混合资源 ID 不冲突 ---

    #[test]
    fn mixed_resources_share_id_space() {
        let rm = ResManager::new(0);
        let font_id = rm.add_font(&CT_Font::new(0, "SimSun"));
        let img_id = rm.add_image_by_path("a.png", "PNG");
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(1.0);
        let dp_id = rm.add_draw_param(&dp);

        assert_ne!(font_id, img_id);
        assert_ne!(font_id, dp_id);
        assert_ne!(img_id, dp_id);
    }

    // --- hash_string 一致性 ---

    #[test]
    fn hash_string_deterministic() {
        let h1 = hash_string("hello");
        let h2 = hash_string("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_string_different_inputs() {
        let h1 = hash_string("hello");
        let h2 = hash_string("world");
        assert_ne!(h1, h2);
    }
}
