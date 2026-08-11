//! CT_PageArea 页面区域结构。

/// 对应 Java: org.ofdrw.core.basicStructure.doc.CT_PageArea
///
/// 页面区域结构，描述页面的物理区域、出血区域、裁切区域等。
/// 对应 GB/T 33190-2016 图 7。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_PageArea {
    /// 物理区域，格式 "topLeftX topLeftY width height"（单位 mm）。
    pub physical_box: Option<String>,
    /// 出版区域（应用显示区域），格式同上。
    pub application_box: Option<String>,
    /// 内容区域（版心），格式同上。
    pub content_box: Option<String>,
    /// 出血区域，格式同上。
    pub bleed_box: Option<String>,
}

/// 由四个值组成的矩形框。
#[derive(Debug, Clone, Copy)]
pub struct Box {
    /// 左上角 X 坐标（mm）。
    pub x: f64,
    /// 左上角 Y 坐标（mm）。
    pub y: f64,
    /// 宽度（mm）。
    pub width: f64,
    /// 高度（mm）。
    pub height: f64,
}

impl Box {
    /// 创建新的矩形框。
    #[must_use]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 转为 OFD ST_Box 格式字符串 "x y width height"。
    #[must_use]
    pub fn to_st_box(&self) -> String {
        format!("{} {} {} {}", self.x, self.y, self.width, self.height)
    }
}

impl CT_PageArea {
    /// 创建空的页面区域。
    #[must_use]
    pub fn new() -> Self {
        Self {
            physical_box: None,
            application_box: None,
            content_box: None,
            bleed_box: None,
        }
    }

    /// 使用物理区域创建页面区域。
    #[must_use]
    pub fn with_physical(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            physical_box: Some(Box::new(x, y, width, height).to_st_box()),
            application_box: None,
            content_box: None,
            bleed_box: None,
        }
    }

    /// 设置物理区域。
    #[must_use]
    pub fn physical_box(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.physical_box = Some(Box::new(x, y, width, height).to_st_box());
        self
    }

    /// 设置出版区域。
    #[must_use]
    pub fn application_box(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.application_box = Some(Box::new(x, y, width, height).to_st_box());
        self
    }

    /// 设置内容区域。
    #[must_use]
    pub fn content_box(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.content_box = Some(Box::new(x, y, width, height).to_st_box());
        self
    }

    /// 设置出血区域。
    #[must_use]
    pub fn bleed_box(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.bleed_box = Some(Box::new(x, y, width, height).to_st_box());
        self
    }

    /// 获取物理区域，若未设置则返回默认值。
    #[must_use]
    pub fn get_box(&self) -> String {
        self.physical_box
            .clone()
            .unwrap_or_else(|| "0 0 210 297".to_string())
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = String::from("<ofd:PageArea");
        if let Some(ref pb) = self.physical_box {
            let _ = write!(xml, " PhysicalBox=\"{pb}\"");
        }
        if let Some(ref ab) = self.application_box {
            let _ = write!(xml, " ApplicationBox=\"{ab}\"");
        }
        if let Some(ref cb) = self.content_box {
            let _ = write!(xml, " ContentBox=\"{cb}\"");
        }
        if let Some(ref bb) = self.bleed_box {
            let _ = write!(xml, " BleedBox=\"{bb}\"");
        }
        xml.push_str(" />");
        xml
    }
}

impl Default for CT_PageArea {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_page_area_new() {
        let area = CT_PageArea::new();
        assert!(area.physical_box.is_none());
        assert!(area.application_box.is_none());
        assert!(area.content_box.is_none());
        assert!(area.bleed_box.is_none());
    }

    #[test]
    fn test_ct_page_area_with_physical() {
        let area = CT_PageArea::with_physical(0.0, 0.0, 210.0, 297.0);
        assert_eq!(area.physical_box.as_deref(), Some("0 0 210 297"));
        assert!(area.application_box.is_none());
    }

    #[test]
    fn test_ct_page_area_builder_chaining() {
        let area = CT_PageArea::new()
            .physical_box(0.0, 0.0, 210.0, 297.0)
            .application_box(10.0, 10.0, 190.0, 277.0)
            .content_box(20.0, 20.0, 170.0, 257.0)
            .bleed_box(-5.0, -5.0, 220.0, 307.0);
        assert!(area.physical_box.is_some());
        assert!(area.application_box.is_some());
        assert!(area.content_box.is_some());
        assert!(area.bleed_box.is_some());
    }

    #[test]
    fn test_ct_page_area_get_box_default() {
        let area = CT_PageArea::new();
        assert_eq!(area.get_box(), "0 0 210 297");
    }

    #[test]
    fn test_ct_page_area_get_box_set() {
        let area = CT_PageArea::with_physical(10.0, 20.0, 100.0, 200.0);
        assert_eq!(area.get_box(), "10 20 100 200");
    }

    #[test]
    fn test_ct_page_area_to_xml_basic() {
        let area = CT_PageArea::with_physical(0.0, 0.0, 210.0, 297.0);
        let xml = area.to_xml_string();
        assert!(xml.contains("<ofd:PageArea"));
        assert!(xml.contains("PhysicalBox=\"0 0 210 297\""));
        assert!(xml.ends_with(" />"));
    }

    #[test]
    fn test_ct_page_area_to_xml_full() {
        let area = CT_PageArea::new()
            .physical_box(0.0, 0.0, 210.0, 297.0)
            .bleed_box(-5.0, -5.0, 220.0, 307.0);
        let xml = area.to_xml_string();
        assert!(xml.contains("PhysicalBox"));
        assert!(xml.contains("BleedBox"));
    }

    #[test]
    fn test_ct_page_area_clone_debug() {
        let area = CT_PageArea::with_physical(0.0, 0.0, 1.0, 1.0);
        let area2 = area.clone();
        assert!(area2.physical_box.is_some());
        assert!(format!("{area:?}").contains("CT_PageArea"));
    }

    #[test]
    fn test_box_new_and_to_st_box() {
        let b = Box::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(b.to_st_box(), "1 2 3 4");
    }
}
