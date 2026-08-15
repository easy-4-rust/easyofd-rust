//! CT_GraphicUnit 图形单元基类。

/// 对应 Java: org.ofdrw.core.pageDescription.CT_GraphicUnit
///
/// 图元对象是版式文档中页面上呈现内容的最基本单元。
/// 所有页面显示内容（文字、图形、图像等）都属于图元对象，
/// 或是图元对象的组合。对应 GB/T 33190-2016 第 8.5 节图 45 表 34。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_GraphicUnit {
    /// 对象 ID，在页面内唯一。
    pub id: u32,
    /// 对象边界框 "topLeftX topLeftY width height"（单位 mm）。
    pub boundary: String,
    /// 对象名称（可选），用于标识图元。
    pub name: Option<String>,
    /// 可见性。true 表示可见（默认），false 表示隐藏。
    pub visible: bool,
    /// 变换矩阵（可选），6 个元素的仿射变换矩阵。
    pub ctm: Option<[f64; 6]>,
    /// 绘制参数引用 ID（可选）。
    pub draw_param: Option<u32>,
    /// 线宽（mm）。
    pub line_width: Option<f64>,
    /// 线端帽类型。
    pub cap: Option<LineCapType>,
    /// 线连接类型。
    pub join: Option<LineJoinType>,
    /// 斜接限制。
    pub miter_limit: Option<f64>,
    /// 虚线偏移。
    pub dash_offset: Option<f64>,
    /// 虚线模式（如 "4 2" 表示 4mm 实线 2mm 间隔）。
    pub dash_pattern: Option<String>,
    /// 透明度 (0-255)，255 表示完全不透明。
    pub alpha: Option<u8>,
}

/// 线端帽类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCapType {
    /// 平头（默认）。
    Butt,
    /// 圆头。
    Round,
    /// 方头。
    Square,
}

impl LineCapType {
    /// 转为 OFD XML 属性值。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Butt => "Butt",
            Self::Round => "Round",
            Self::Square => "Square",
        }
    }
}

impl std::fmt::Display for LineCapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 线连接类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoinType {
    /// 尖角（默认）。
    Miter,
    /// 圆角。
    Round,
    /// 平角。
    Bevel,
}

impl LineJoinType {
    /// 转为 OFD XML 属性值。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Miter => "Miter",
            Self::Round => "Round",
            Self::Bevel => "Bevel",
        }
    }
}

impl std::fmt::Display for LineJoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CT_GraphicUnit {
    /// 创建新的图形单元。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            name: None,
            visible: true,
            ctm: None,
            draw_param: None,
            line_width: None,
            cap: None,
            join: None,
            miter_limit: None,
            dash_offset: None,
            dash_pattern: None,
            alpha: None,
        }
    }

    /// 设置对象名称。
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置可见性。
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// 设置变换矩阵。
    #[must_use]
    pub fn ctm(mut self, ctm: [f64; 6]) -> Self {
        self.ctm = Some(ctm);
        self
    }

    /// 设置绘制参数引用。
    #[must_use]
    pub fn draw_param(mut self, id: u32) -> Self {
        self.draw_param = Some(id);
        self
    }

    /// 设置线宽。
    #[must_use]
    pub fn line_width(mut self, width: f64) -> Self {
        self.line_width = Some(width);
        self
    }

    /// 设置线端帽类型。
    #[must_use]
    pub fn cap(mut self, cap: LineCapType) -> Self {
        self.cap = Some(cap);
        self
    }

    /// 设置线连接类型。
    #[must_use]
    pub fn join(mut self, join: LineJoinType) -> Self {
        self.join = Some(join);
        self
    }

    /// 设置斜接限制。
    #[must_use]
    pub fn miter_limit(mut self, limit: f64) -> Self {
        self.miter_limit = Some(limit);
        self
    }

    /// 设置虚线偏移。
    #[must_use]
    pub fn dash_offset(mut self, offset: f64) -> Self {
        self.dash_offset = Some(offset);
        self
    }

    /// 设置虚线模式。
    #[must_use]
    pub fn dash_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.dash_pattern = Some(pattern.into());
        self
    }

    /// 设置透明度 (0-255)。
    #[must_use]
    pub fn alpha(mut self, alpha: u8) -> Self {
        self.alpha = Some(alpha);
        self
    }

    /// 获取对象 ID。
    #[must_use]
    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// 获取边界框。
    #[must_use]
    pub fn get_boundary(&self) -> &str {
        &self.boundary
    }

    /// 获取对象名称。
    #[must_use]
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 获取可见性。
    #[must_use]
    pub fn get_visible(&self) -> bool {
        self.visible
    }

    /// 获取变换矩阵。
    #[must_use]
    pub fn get_ctm(&self) -> Option<[f64; 6]> {
        self.ctm
    }

    /// 获取透明度。
    #[must_use]
    pub fn get_alpha(&self) -> Option<u8> {
        self.alpha
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = format!(
            "<ofd:CT_GraphicUnit ID=\"{}\" Boundary=\"{}\"",
            self.id, self.boundary
        );
        if let Some(ref name) = self.name {
            write!(xml, " Name=\"{name}\"").expect("写入内存缓冲区不会失败");
        }
        if !self.visible {
            xml.push_str(" Visible=\"false\"");
        }
        if let Some(ctm) = self.ctm {
            write!(
                xml,
                " CTM=\"{} {} {} {} {} {}\"",
                ctm[0], ctm[1], ctm[2], ctm[3], ctm[4], ctm[5]
            )
            .expect("写入内存缓冲区不会失败");
        }
        if let Some(dp) = self.draw_param {
            write!(xml, " DrawParam=\"{dp}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(lw) = self.line_width {
            write!(xml, " LineWidth=\"{lw}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(cap) = self.cap {
            write!(xml, " Cap=\"{}\"", cap.as_str()).expect("写入内存缓冲区不会失败");
        }
        if let Some(join) = self.join {
            write!(xml, " Join=\"{}\"", join.as_str()).expect("写入内存缓冲区不会失败");
        }
        if let Some(ml) = self.miter_limit {
            write!(xml, " MiterLimit=\"{ml}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(doff) = self.dash_offset {
            write!(xml, " DashOffset=\"{doff}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(ref dp) = self.dash_pattern {
            write!(xml, " DashPattern=\"{dp}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(a) = self.alpha {
            write!(xml, " Alpha=\"{a}\"").expect("写入内存缓冲区不会失败");
        }
        xml.push_str(" />");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_graphic_unit_new() {
        let gu = CT_GraphicUnit::new(1, "0 0 100 50");
        assert_eq!(gu.id, 1);
        assert_eq!(gu.boundary, "0 0 100 50");
        assert!(gu.name.is_none());
        assert!(gu.visible);
        assert!(gu.ctm.is_none());
        assert!(gu.alpha.is_none());
    }

    #[test]
    fn test_ct_graphic_unit_builder_chaining() {
        let gu = CT_GraphicUnit::new(2, "10 20 50 50")
            .name("rect1")
            .visible(false)
            .line_width(1.5)
            .cap(LineCapType::Round)
            .join(LineJoinType::Bevel)
            .alpha(128);
        assert_eq!(gu.get_name(), Some("rect1"));
        assert!(!gu.get_visible());
        assert!((gu.line_width.unwrap() - 1.5).abs() < f64::EPSILON);
        assert_eq!(gu.cap, Some(LineCapType::Round));
        assert_eq!(gu.join, Some(LineJoinType::Bevel));
        assert_eq!(gu.get_alpha(), Some(128));
    }

    #[test]
    fn test_ct_graphic_unit_ctm() {
        let gu = CT_GraphicUnit::new(3, "0 0 10 10").ctm([1.0, 0.0, 0.0, 1.0, 5.0, 5.0]);
        let ctm = gu.get_ctm().unwrap();
        assert!((ctm[4] - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_line_cap_type_display() {
        assert_eq!(LineCapType::Butt.to_string(), "Butt");
        assert_eq!(LineCapType::Round.to_string(), "Round");
        assert_eq!(LineCapType::Square.to_string(), "Square");
    }

    #[test]
    fn test_line_join_type_display() {
        assert_eq!(LineJoinType::Miter.to_string(), "Miter");
        assert_eq!(LineJoinType::Round.to_string(), "Round");
        assert_eq!(LineJoinType::Bevel.to_string(), "Bevel");
    }

    #[test]
    fn test_ct_graphic_unit_to_xml_minimal() {
        let gu = CT_GraphicUnit::new(1, "0 0 100 50");
        let xml = gu.to_xml_string();
        assert!(xml.contains("ID=\"1\""));
        assert!(xml.contains("Boundary=\"0 0 100 50\""));
        assert!(xml.contains("<ofd:CT_GraphicUnit"));
        assert!(xml.ends_with(" />"));
    }

    #[test]
    fn test_ct_graphic_unit_to_xml_full() {
        let gu = CT_GraphicUnit::new(5, "0 0 200 100")
            .name("myUnit")
            .visible(false)
            .ctm([2.0, 0.0, 0.0, 2.0, 10.0, 20.0])
            .line_width(0.5)
            .cap(LineCapType::Square)
            .join(LineJoinType::Miter)
            .miter_limit(4.0)
            .dash_offset(1.0)
            .dash_pattern("4 2")
            .alpha(200);
        let xml = gu.to_xml_string();
        assert!(xml.contains("Name=\"myUnit\""));
        assert!(xml.contains("Visible=\"false\""));
        assert!(xml.contains("CTM=\"2 0 0 2 10 20\""));
        assert!(xml.contains("LineWidth=\"0.5\""));
        assert!(xml.contains("Cap=\"Square\""));
        assert!(xml.contains("Join=\"Miter\""));
        assert!(xml.contains("MiterLimit=\"4\""));
        assert!(xml.contains("DashOffset=\"1\""));
        assert!(xml.contains("DashPattern=\"4 2\""));
        assert!(xml.contains("Alpha=\"200\""));
    }

    #[test]
    fn test_ct_graphic_unit_clone_debug() {
        let gu = CT_GraphicUnit::new(1, "0 0 1 1");
        let gu2 = gu.clone();
        assert_eq!(gu2.id, 1);
        assert!(format!("{gu:?}").contains("CT_GraphicUnit"));
    }
}
