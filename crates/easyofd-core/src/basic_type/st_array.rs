//! 数组类型。
//!
//! 对应 Java: org.ofdrw.core.basicType.ST_Array

/// 数组，以空格来分割元素。元素可以是除 ST_Loc、ST_Array 外的数据类型，不可嵌套。
///
/// 示例：`1 2.0 5.0`
///
/// 对应 Java: org.ofdrw.core.basicType.ST_Array
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct ST_Array {
    /// 元素列表（以字符串存储）
    array: Vec<String>,
}

impl ST_Array {
    /// 创建空数组。
    pub fn new() -> Self {
        Self { array: Vec::new() }
    }

    /// 从字符串列表创建。
    pub fn from_values(values: Vec<String>) -> Self {
        Self { array: values }
    }

    /// 创建变换矩阵（6 个元素）。
    #[allow(clippy::many_single_char_names)]
    pub fn transform(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self {
            array: vec![
                a.to_string(),
                b.to_string(),
                c.to_string(),
                d.to_string(),
                e.to_string(),
                f.to_string(),
            ],
        }
    }

    /// 添加元素。
    pub fn push(&mut self, item: &str) {
        self.array.push(item.to_string());
    }

    /// 添加数值元素。
    pub fn push_number(&mut self, val: f64) {
        self.array.push(val.to_string());
    }

    /// 获取元素列表。
    pub fn array(&self) -> &[String] {
        &self.array
    }

    /// 元素个数。
    pub fn len(&self) -> usize {
        self.array.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    /// 获取指定索引的值（解析为 f64）。
    pub fn get_f64(&self, index: usize) -> Option<f64> {
        self.array.get(index)?.parse().ok()
    }

    /// 获取指定索引的值（解析为 i64）。
    pub fn get_i64(&self, index: usize) -> Option<i64> {
        self.array.get(index)?.parse().ok()
    }

    /// 转换为 f64 数组。
    pub fn to_f64_vec(&self) -> Vec<f64> {
        self.array.iter().filter_map(|s| s.parse().ok()).collect()
    }

    /// 转换为 3x3 矩阵。
    pub fn to_matrix(&self) -> Option<[[f64; 3]; 3]> {
        if self.array.len() != 6 {
            return None;
        }
        let d = self.to_f64_vec();
        if d.len() != 6 {
            return None;
        }
        Some([[d[0], d[1], 0.0], [d[2], d[3], 0.0], [d[4], d[5], 1.0]])
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        self.array.join(" ")
    }

    /// 从字符串解析 ST_Array。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let array: Vec<String> = s
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(Self { array })
    }
}

impl Default for ST_Array {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::xml_element::XmlElement for ST_Array {
    /// 对应 Java: ST_Array 元素名 "ST_Array"。
    fn element_name(&self) -> &'static str {
        "ST_Array"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml 以处理计算得到的文本内容。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<ST_Array>");
        if !self.array.is_empty() {
            out.push_str(&crate::xml_element::xml_escape(&self.array.join(" ")));
        }
        out.push_str("</ST_Array>");
    }

    fn from_xml(
        node: &crate::xml_element::XmlNode,
    ) -> Result<Self, crate::xml_element::XmlElementError> {
        let text = node.text.as_deref().ok_or_else(|| {
            crate::xml_element::XmlElementError("ST_Array 缺少文本内容".to_string())
        })?;
        Self::from_str(text)
            .map_err(|e| crate::xml_element::XmlElementError(format!("解析 ST_Array 失败: {e}")))
    }
}

impl ST_Array {
    /// 生成包含文本子节点的 XmlNode（供嵌套序列化）。
    pub fn to_xml_node(&self) -> crate::xml_element::XmlNode {
        let mut node = crate::xml_element::XmlNode::element("ST_Array");
        if !self.array.is_empty() {
            node.text = Some(self.array.join(" "));
        }
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_xml_string() {
        let mut arr = ST_Array::new();
        arr.push("1");
        arr.push("2.0");
        arr.push("5.0");
        assert_eq!(arr.to_xml_string(), "1 2.0 5.0");
    }

    #[test]
    fn test_from_str() {
        let arr = ST_Array::from_str("1 2.0 5.0").unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_f64(0), Some(1.0));
        assert_eq!(arr.get_f64(1), Some(2.0));
        assert_eq!(arr.get_f64(2), Some(5.0));
    }

    #[test]
    fn test_transform() {
        let arr = ST_Array::transform(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
        assert_eq!(arr.len(), 6);
        assert_eq!(arr.to_xml_string(), "1 0 0 1 10 20");
    }

    #[test]
    fn test_to_matrix() {
        let arr = ST_Array::transform(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
        let m = arr.to_matrix().unwrap();
        assert_eq!(m[0], [1.0, 0.0, 0.0]);
        assert_eq!(m[1], [0.0, 1.0, 0.0]);
        assert_eq!(m[2], [10.0, 20.0, 1.0]);
    }

    #[test]
    fn test_roundtrip() {
        let arr = ST_Array::from_str("100 200 300").unwrap();
        let s = arr.to_xml_string();
        let arr2 = ST_Array::from_str(&s).unwrap();
        assert_eq!(arr, arr2);
    }

    #[test]
    fn test_is_empty() {
        let arr = ST_Array::new();
        assert!(arr.is_empty());
    }

    #[test]
    fn test_xml_element_roundtrip() {
        use crate::xml_element::XmlElement;
        use crate::xml_parse::parse_xml_to_nodes;
        let arr = ST_Array::from_str("1 2.5 3").unwrap();
        let xml = arr.to_xml();
        assert!(xml.contains("<ST_Array>"));
        assert!(xml.contains("1 2.5 3"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let arr2 = ST_Array::from_xml(&node).unwrap();
        assert_eq!(arr, arr2);
    }

    #[test]
    fn test_xml_element_name() {
        use crate::xml_element::XmlElement;
        let arr = ST_Array::new();
        assert_eq!(arr.element_name(), "ST_Array");
    }
}
