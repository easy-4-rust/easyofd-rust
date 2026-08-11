//! 文件路径定位器类型。
//!
//! 对应 Java: org.ofdrw.core.basicType.ST_Loc

/// 文件路径定位器，用于定位 OFD 包内的资源文件。
///
/// 对应 Java: org.ofdrw.core.basicType.ST_Loc
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ST_Loc {
    /// 路径字符串
    loc: String,
}

impl ST_Loc {
    /// 创建新的路径定位器。
    pub fn new(path: &str) -> Self {
        Self {
            loc: path.to_string(),
        }
    }

    /// 获取路径字符串。
    pub fn loc(&self) -> &str {
        &self.loc
    }

    /// 设置路径。
    pub fn set_loc(&mut self, loc: &str) {
        self.loc = loc.to_string();
    }

    /// 按 '/' 分割路径。
    pub fn split(&self) -> Vec<&str> {
        self.loc.split('/').collect()
    }

    /// 获取路径的各部分（过滤空字符串）。
    pub fn parts(&self) -> Vec<&str> {
        self.loc.split('/').filter(|s| !s.is_empty()).collect()
    }

    /// 获取父路径。
    pub fn parent(&self) -> Option<String> {
        let parts = self.parts();
        if parts.len() <= 1 {
            None
        } else {
            Some(parts[..parts.len() - 1].join("/"))
        }
    }

    /// 获取父级定位器。
    pub fn parent_loc(&self) -> Option<ST_Loc> {
        self.parent().map(|p| ST_Loc::new(&p))
    }

    /// 获取文件名（路径最后一部分）。
    pub fn file_name(&self) -> Option<&str> {
        self.parts().last().copied()
    }

    /// 连接路径。
    pub fn cat(&self, other: &str) -> ST_Loc {
        if self.loc.is_empty() {
            ST_Loc::new(other)
        } else if other.is_empty() {
            self.clone()
        } else {
            ST_Loc::new(&format!(
                "{}/{}",
                self.loc.trim_end_matches('/'),
                other.trim_start_matches('/')
            ))
        }
    }

    /// 连接另一个定位器。
    pub fn cat_loc(&self, other: &ST_Loc) -> ST_Loc {
        self.cat(other.loc())
    }

    /// 是否以指定后缀结尾。
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.loc.ends_with(suffix)
    }

    /// 是否以指定前缀开头。
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.loc.starts_with(prefix)
    }

    /// 是否为根路径。
    pub fn is_root_path(&self) -> bool {
        self.loc.starts_with('/')
    }

    /// 是否为空路径。
    pub fn is_empty(&self) -> bool {
        self.loc.is_empty()
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        self.loc.clone()
    }

    /// 从字符串解析 ST_Loc。
    pub fn from_str(s: &str) -> Result<Self, String> {
        Ok(Self::new(s.trim()))
    }
}

impl std::fmt::Display for ST_Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.loc)
    }
}

impl crate::xml_element::XmlElement for ST_Loc {
    /// 对应 Java: ST_Loc 元素名 "ST_Loc"。
    fn element_name(&self) -> &'static str {
        "ST_Loc"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml 以处理计算得到的文本内容。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<ST_Loc>");
        out.push_str(&crate::xml_element::xml_escape(&self.loc));
        out.push_str("</ST_Loc>");
    }

    fn from_xml(
        node: &crate::xml_element::XmlNode,
    ) -> Result<Self, crate::xml_element::XmlElementError> {
        let text = node.text.as_deref().ok_or_else(|| {
            crate::xml_element::XmlElementError("ST_Loc 缺少文本内容".to_string())
        })?;
        Ok(Self::new(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_path() {
        let loc = ST_Loc::new("Doc_0/Res/image.png");
        assert_eq!(loc.loc(), "Doc_0/Res/image.png");
        assert_eq!(loc.file_name(), Some("image.png"));
    }

    #[test]
    fn test_parent() {
        let loc = ST_Loc::new("Doc_0/Res/image.png");
        let parent = loc.parent().unwrap();
        assert_eq!(parent, "Doc_0/Res");
    }

    #[test]
    fn test_parts() {
        let loc = ST_Loc::new("/Doc_0/Res/image.png");
        let parts = loc.parts();
        assert_eq!(parts, vec!["Doc_0", "Res", "image.png"]);
    }

    #[test]
    fn test_cat() {
        let base = ST_Loc::new("Doc_0/Res");
        let joined = base.cat("image.png");
        assert_eq!(joined.loc(), "Doc_0/Res/image.png");
    }

    #[test]
    fn test_to_xml_string() {
        let loc = ST_Loc::new("Doc_0/Content.xml");
        assert_eq!(loc.to_xml_string(), "Doc_0/Content.xml");
    }

    #[test]
    fn test_from_str() {
        let loc = ST_Loc::from_str("Doc_0/Res").unwrap();
        assert_eq!(loc.loc(), "Doc_0/Res");
    }

    #[test]
    fn test_ends_with() {
        let loc = ST_Loc::new("Doc_0/Res/image.png");
        assert!(loc.ends_with(".png"));
        assert!(!loc.ends_with(".jpg"));
    }

    #[test]
    fn test_starts_with() {
        let loc = ST_Loc::new("Doc_0/Res/image.png");
        assert!(loc.starts_with("Doc_0"));
    }

    #[test]
    fn test_is_empty() {
        let empty = ST_Loc::new("");
        assert!(empty.is_empty());
        let non_empty = ST_Loc::new("test");
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_xml_element_roundtrip() {
        use crate::xml_element::XmlElement;
        use crate::xml_parse::parse_xml_to_nodes;
        let loc = ST_Loc::new("Doc_0/Res/image.png");
        let xml = loc.to_xml();
        assert!(xml.contains("<ST_Loc>"));
        assert!(xml.contains("Doc_0/Res/image.png"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let loc2 = ST_Loc::from_xml(&node).unwrap();
        assert_eq!(loc, loc2);
    }

    #[test]
    fn test_xml_element_name() {
        use crate::xml_element::XmlElement;
        let loc = ST_Loc::new("test");
        assert_eq!(loc.element_name(), "ST_Loc");
    }
}
