//! CT_CGTransform 字形坐标变换。

/// 对应 Java: org.ofdrw.core.text.CT_CGTransform
///
/// 变换描述。当存在字形变换时，TextCode 对象中使用 CGTransform 节点
/// 描述字符编码和字形索引之间的关系。
/// 对应 GB/T 33190-2016 第 11.4.1 节图 66 表 48。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_CGTransform {
    /// 在 TextCode 文本内容中的起始位置（从 0 开始）。
    pub code_position: Option<u32>,
    /// 参与变换的字符数量。
    pub code_count: Option<u32>,
    /// 对应的字形数量。
    pub glyph_count: Option<u32>,
    /// 字形索引列表。
    pub glyphs: Vec<u32>,
}

impl CT_CGTransform {
    /// 创建空的字形变换。
    #[must_use]
    pub fn new() -> Self {
        Self {
            code_position: None,
            code_count: None,
            glyph_count: None,
            glyphs: Vec::new(),
        }
    }

    /// 设置字符起始位置。
    #[must_use]
    pub fn code_position(mut self, pos: u32) -> Self {
        self.code_position = Some(pos);
        self
    }

    /// 设置字符数量。
    #[must_use]
    pub fn code_count(mut self, count: u32) -> Self {
        self.code_count = Some(count);
        self
    }

    /// 设置字形数量。
    #[must_use]
    pub fn glyph_count(mut self, count: u32) -> Self {
        self.glyph_count = Some(count);
        self
    }

    /// 设置字形索引列表。
    #[must_use]
    pub fn glyphs(mut self, glyphs: Vec<u32>) -> Self {
        self.glyphs = glyphs;
        self
    }

    /// 添加字形索引。
    pub fn add_glyph(&mut self, glyph: u32) {
        self.glyphs.push(glyph);
    }

    /// 获取字符起始位置。
    #[must_use]
    pub fn get_code_position(&self) -> Option<u32> {
        self.code_position
    }

    /// 获取字符数量。
    #[must_use]
    pub fn get_code_count(&self) -> Option<u32> {
        self.code_count
    }

    /// 获取字形数量。
    #[must_use]
    pub fn get_glyph_count(&self) -> Option<u32> {
        self.glyph_count
    }

    /// 获取字形索引列表。
    #[must_use]
    pub fn get_glyphs(&self) -> &[u32] {
        &self.glyphs
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = String::from("<ofd:CGTransform");
        if let Some(cp) = self.code_position {
            write!(xml, " CodePosition=\"{cp}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(cc) = self.code_count {
            write!(xml, " CodeCount=\"{cc}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(gc) = self.glyph_count {
            write!(xml, " GlyphCount=\"{gc}\"").expect("写入内存缓冲区不会失败");
        }
        if !self.glyphs.is_empty() {
            xml.push_str(" Glyphs=\"");
            for (i, g) in self.glyphs.iter().enumerate() {
                if i > 0 {
                    xml.push(' ');
                }
                write!(xml, "{g}").expect("写入内存缓冲区不会失败");
            }
            xml.push('"');
        }
        xml.push_str(" />");
        xml
    }
}

impl Default for CT_CGTransform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_cg_transform_new() {
        let t = CT_CGTransform::new();
        assert!(t.code_position.is_none());
        assert!(t.code_count.is_none());
        assert!(t.glyph_count.is_none());
        assert!(t.glyphs.is_empty());
    }

    #[test]
    fn test_ct_cg_transform_builder() {
        let t = CT_CGTransform::new()
            .code_position(0)
            .code_count(2)
            .glyph_count(2)
            .glyphs(vec![100, 200]);
        assert_eq!(t.get_code_position(), Some(0));
        assert_eq!(t.get_code_count(), Some(2));
        assert_eq!(t.get_glyph_count(), Some(2));
        assert_eq!(t.get_glyphs(), &[100, 200]);
    }

    #[test]
    fn test_ct_cg_transform_add_glyph() {
        let mut t = CT_CGTransform::new();
        t.add_glyph(50);
        t.add_glyph(60);
        t.add_glyph(70);
        assert_eq!(t.get_glyphs().len(), 3);
    }

    #[test]
    fn test_ct_cg_transform_to_xml_minimal() {
        let t = CT_CGTransform::new();
        let xml = t.to_xml_string();
        assert!(xml.contains("<ofd:CGTransform"));
        assert!(xml.ends_with(" />"));
    }

    #[test]
    fn test_ct_cg_transform_to_xml_full() {
        let t = CT_CGTransform::new()
            .code_position(3)
            .code_count(1)
            .glyph_count(2)
            .glyphs(vec![101, 202]);
        let xml = t.to_xml_string();
        assert!(xml.contains("CodePosition=\"3\""));
        assert!(xml.contains("CodeCount=\"1\""));
        assert!(xml.contains("GlyphCount=\"2\""));
        assert!(xml.contains("Glyphs=\"101 202\""));
    }

    #[test]
    fn test_ct_cg_transform_clone_debug() {
        let t = CT_CGTransform::new().code_position(1);
        let t2 = t.clone();
        assert_eq!(t2.get_code_position(), Some(1));
        assert!(format!("{t:?}").contains("CT_CGTransform"));
    }
}
