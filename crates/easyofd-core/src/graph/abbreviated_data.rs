//! AbbreviatedData 路径缩写数据。

use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 路径操作命令类型。
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    /// 移动到 (x, y)。
    M(f64, f64),
    /// 直线到 (x, y)。
    L(f64, f64),
    /// 二次贝塞尔曲线 (cx, cy, x, y)。
    Q(f64, f64, f64, f64),
    /// 三次贝塞尔曲线 (c1x, c1y, c2x, c2y, x, y)。
    B(f64, f64, f64, f64, f64, f64),
    /// 椭圆弧 (rx, ry, angle, large, sweep, x, y)。
    A(f64, f64, f64, i32, i32, f64, f64),
    /// 闭合路径。
    C,
}

/// 对应 Java: org.ofdrw.core.graph.pathObj.AbbreviatedData
///
/// 图形轮廓数据，由一系列紧缩的操作符和操作数构成。
/// 支持 M(移动)、L(直线)、Q(二次贝塞尔)、B(三次贝塞尔)、
/// A(椭圆弧)、C(闭合) 命令。
/// 对应 GB/T 33190-2016 第 9.1 节表 35-36。
#[derive(Debug, Clone)]
pub struct AbbreviatedData {
    /// 命令序列。
    pub commands: Vec<PathCommand>,
}

impl AbbreviatedData {
    /// 创建空的缩写数据。
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// 从命令字符串解析缩写数据。
    ///
    /// 支持格式: "M x y L x y Q cx cy x y B c1x c1y c2x c2y x y A rx ry angle large sweep x y C"
    #[must_use]
    pub fn parse(data: &str) -> Self {
        let mut commands = Vec::new();
        let mut chars = data.chars().peekable();
        while let Some(&ch) = chars.peek() {
            match ch {
                'M' | 'm' => {
                    chars.next();
                    if let (Some(x), Some(y)) = (next_f64(&mut chars), next_f64(&mut chars)) {
                        commands.push(PathCommand::M(x, y));
                    }
                }
                'L' | 'l' => {
                    chars.next();
                    if let (Some(x), Some(y)) = (next_f64(&mut chars), next_f64(&mut chars)) {
                        commands.push(PathCommand::L(x, y));
                    }
                }
                'Q' | 'q' => {
                    chars.next();
                    if let (Some(cx), Some(cy), Some(x), Some(y)) = (
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                    ) {
                        commands.push(PathCommand::Q(cx, cy, x, y));
                    }
                }
                'B' | 'b' => {
                    chars.next();
                    if let (Some(c1x), Some(c1y), Some(c2x), Some(c2y), Some(x), Some(y)) = (
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                    ) {
                        commands.push(PathCommand::B(c1x, c1y, c2x, c2y, x, y));
                    }
                }
                'A' | 'a' => {
                    chars.next();
                    if let (
                        Some(rx),
                        Some(ry),
                        Some(angle),
                        Some(large),
                        Some(sweep),
                        Some(x),
                        Some(y),
                    ) = (
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                        next_i32(&mut chars),
                        next_i32(&mut chars),
                        next_f64(&mut chars),
                        next_f64(&mut chars),
                    ) {
                        commands.push(PathCommand::A(rx, ry, angle, large, sweep, x, y));
                    }
                }
                'C' | 'c' => {
                    chars.next();
                    commands.push(PathCommand::C);
                }
                _ => {
                    // Skip whitespace or unknown chars.
                    chars.next();
                }
            }
        }
        Self { commands }
    }

    /// 添加移动命令。
    #[must_use]
    pub fn move_to(mut self, x: f64, y: f64) -> Self {
        self.commands.push(PathCommand::M(x, y));
        self
    }

    /// 添加直线命令。
    #[must_use]
    pub fn line_to(mut self, x: f64, y: f64) -> Self {
        self.commands.push(PathCommand::L(x, y));
        self
    }

    /// 添加二次贝塞尔曲线命令。
    #[must_use]
    pub fn quad_to(mut self, cx: f64, cy: f64, x: f64, y: f64) -> Self {
        self.commands.push(PathCommand::Q(cx, cy, x, y));
        self
    }

    /// 添加三次贝塞尔曲线命令。
    #[must_use]
    pub fn cubic_to(mut self, c1x: f64, c1y: f64, c2x: f64, c2y: f64, x: f64, y: f64) -> Self {
        self.commands.push(PathCommand::B(c1x, c1y, c2x, c2y, x, y));
        self
    }

    /// 添加椭圆弧命令。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn arc_to(
        mut self,
        rx: f64,
        ry: f64,
        angle: f64,
        large: i32,
        sweep: i32,
        x: f64,
        y: f64,
    ) -> Self {
        self.commands
            .push(PathCommand::A(rx, ry, angle, large, sweep, x, y));
        self
    }

    /// 添加闭合命令。
    #[must_use]
    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::C);
        self
    }

    /// 命令数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// 追加另一组缩写数据。
    pub fn append(&mut self, other: &AbbreviatedData) {
        self.commands.extend_from_slice(&other.commands);
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        format!(
            "<ofd:AbbreviatedData>{}</ofd:AbbreviatedData>",
            self.to_data_string()
        )
    }

    /// 序列化为路径数据字符串。
    #[must_use]
    pub fn to_data_string(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        for cmd in &self.commands {
            match cmd {
                PathCommand::M(x, y) => write!(s, "M {x} {y} ").unwrap(),
                PathCommand::L(x, y) => write!(s, "L {x} {y} ").unwrap(),
                PathCommand::Q(cx, cy, x, y) => write!(s, "Q {cx} {cy} {x} {y} ").unwrap(),
                PathCommand::B(c1x, c1y, c2x, c2y, x, y) => {
                    write!(s, "B {c1x} {c1y} {c2x} {c2y} {x} {y} ")
                        .expect("写入内存缓冲区不会失败");
                }
                PathCommand::A(rx, ry, angle, large, sweep, x, y) => {
                    write!(s, "A {rx} {ry} {angle} {large} {sweep} {x} {y} ")
                        .expect("写入内存缓冲区不会失败");
                }
                PathCommand::C => write!(s, "C ").unwrap(),
            }
        }
        s.trim_end().to_string()
    }
}

impl Default for AbbreviatedData {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlElement for AbbreviatedData {
    /// 对应 Java: AbbreviatedData 元素名 "AbbreviatedData"。
    fn element_name(&self) -> &'static str {
        "AbbreviatedData"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml：文本内容为路径命令字符串。
    fn write_xml(&self, out: &mut String) {
        if self.commands.is_empty() {
            out.push_str("<AbbreviatedData/>");
        } else {
            out.push_str("<AbbreviatedData>");
            out.push_str(&crate::xml_element::xml_escape(&self.to_data_string()));
            out.push_str("</AbbreviatedData>");
        }
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let text = node.text.as_deref().unwrap_or("");
        Ok(Self::parse(text))
    }
}

/// 从字符迭代器中读取下一个 f64 值。
fn next_f64(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<f64> {
    skip_whitespace(chars);
    let mut num = String::new();
    // Handle negative sign.
    if chars.peek() == Some(&'-') || chars.peek() == Some(&'+') {
        num.push(chars.next()?);
    }
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() || ch == '.' {
            num.push(chars.next()?);
        } else {
            break;
        }
    }
    num.parse::<f64>().ok()
}

/// 从字符迭代器中读取下一个 i32 值。
fn next_i32(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<i32> {
    skip_whitespace(chars);
    let mut num = String::new();
    if chars.peek() == Some(&'-') || chars.peek() == Some(&'+') {
        num.push(chars.next()?);
    }
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            num.push(chars.next()?);
        } else {
            break;
        }
    }
    num.parse::<i32>().ok()
}

/// 跳过空白字符。
fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() || ch == ',' {
            chars.next();
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn test_abbreviated_data_new() {
        let ad = AbbreviatedData::new();
        assert!(ad.is_empty());
        assert_eq!(ad.len(), 0);
    }

    #[test]
    fn test_abbreviated_data_builder() {
        let ad = AbbreviatedData::new()
            .move_to(0.0, 0.0)
            .line_to(10.0, 10.0)
            .close();
        assert_eq!(ad.len(), 3);
        assert!(!ad.is_empty());
    }

    #[test]
    fn test_abbreviated_data_parse_simple() {
        let ad = AbbreviatedData::parse("M 0 0 L 10 10 C");
        assert_eq!(ad.len(), 3);
        assert_eq!(ad.commands[0], PathCommand::M(0.0, 0.0));
        assert_eq!(ad.commands[1], PathCommand::L(10.0, 10.0));
        assert_eq!(ad.commands[2], PathCommand::C);
    }

    #[test]
    fn test_abbreviated_data_parse_quad() {
        let ad = AbbreviatedData::parse("M0 0 Q 5 5 10 0");
        assert_eq!(ad.len(), 2);
        assert_eq!(ad.commands[1], PathCommand::Q(5.0, 5.0, 10.0, 0.0));
    }

    #[test]
    fn test_abbreviated_data_parse_cubic() {
        let ad = AbbreviatedData::parse("M0 0 B 2 4 6 8 10 0");
        assert_eq!(ad.len(), 2);
        assert_eq!(
            ad.commands[1],
            PathCommand::B(2.0, 4.0, 6.0, 8.0, 10.0, 0.0)
        );
    }

    #[test]
    fn test_abbreviated_data_parse_arc() {
        let ad = AbbreviatedData::parse("A 5 5 0 1 0 10 10");
        assert_eq!(ad.len(), 1);
        assert_eq!(
            ad.commands[0],
            PathCommand::A(5.0, 5.0, 0.0, 1, 0, 10.0, 10.0)
        );
    }

    #[test]
    fn test_abbreviated_data_parse_negative() {
        let ad = AbbreviatedData::parse("M -1.5 -2.5");
        assert_eq!(ad.len(), 1);
        assert_eq!(ad.commands[0], PathCommand::M(-1.5, -2.5));
    }

    #[test]
    fn test_abbreviated_data_to_data_string() {
        let ad = AbbreviatedData::new()
            .move_to(0.0, 0.0)
            .line_to(100.0, 50.0)
            .close();
        let s = ad.to_data_string();
        assert!(s.contains("M 0 0"));
        assert!(s.contains("L 100 50"));
        assert!(s.contains('C'));
    }

    #[test]
    fn test_abbreviated_data_to_xml_string() {
        let ad = AbbreviatedData::new().move_to(1.0, 2.0);
        let xml = ad.to_xml_string();
        assert!(xml.contains("<ofd:AbbreviatedData>"));
        assert!(xml.contains("</ofd:AbbreviatedData>"));
        assert!(xml.contains("M 1 2"));
    }

    #[test]
    fn test_abbreviated_data_quad_to() {
        let ad = AbbreviatedData::new().quad_to(5.0, 5.0, 10.0, 0.0);
        assert_eq!(ad.len(), 1);
        assert_eq!(ad.commands[0], PathCommand::Q(5.0, 5.0, 10.0, 0.0));
    }

    #[test]
    fn test_abbreviated_data_cubic_to() {
        let ad = AbbreviatedData::new().cubic_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(ad.commands[0], PathCommand::B(1.0, 2.0, 3.0, 4.0, 5.0, 6.0));
    }

    #[test]
    fn test_abbreviated_data_arc_to() {
        let ad = AbbreviatedData::new().arc_to(5.0, 5.0, 0.0, 1, 0, 10.0, 10.0);
        assert_eq!(
            ad.commands[0],
            PathCommand::A(5.0, 5.0, 0.0, 1, 0, 10.0, 10.0)
        );
    }

    #[test]
    fn test_abbreviated_data_append() {
        let mut ad1 = AbbreviatedData::new().move_to(0.0, 0.0);
        let ad2 = AbbreviatedData::new().line_to(10.0, 10.0).close();
        ad1.append(&ad2);
        assert_eq!(ad1.len(), 3);
    }

    #[test]
    fn test_abbreviated_data_roundtrip() {
        let original = "M 0 0 L 10 10 L 20 0 C";
        let ad = AbbreviatedData::parse(original);
        let result = ad.to_data_string();
        assert_eq!(result, original);
    }

    #[test]
    fn test_abbreviated_data_clone_debug() {
        let ad = AbbreviatedData::new().move_to(1.0, 2.0);
        let ad2 = ad.clone();
        assert_eq!(ad2.len(), 1);
        assert!(format!("{ad:?}").contains("AbbreviatedData"));
    }

    #[test]
    fn test_xml_element_name() {
        let ad = AbbreviatedData::new();
        assert_eq!(ad.element_name(), "AbbreviatedData");
    }

    #[test]
    fn test_xml_element_roundtrip() {
        let ad = AbbreviatedData::new()
            .move_to(0.0, 0.0)
            .line_to(10.0, 10.0)
            .quad_to(5.0, 5.0, 20.0, 0.0)
            .cubic_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0)
            .arc_to(5.0, 5.0, 0.0, 1, 0, 10.0, 10.0)
            .close();
        let xml = ad.to_xml();
        assert!(xml.contains("<AbbreviatedData>"));
        assert!(xml.contains("M 0 0"));
        assert!(xml.contains("L 10 10"));
        assert!(xml.contains('C'));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let ad2 = AbbreviatedData::from_xml(&node).unwrap();
        assert_eq!(ad.commands, ad2.commands);
    }

    #[test]
    fn test_xml_element_roundtrip_empty() {
        let ad = AbbreviatedData::new();
        let xml = ad.to_xml();
        assert_eq!(xml, "<AbbreviatedData/>");
        let node = parse_xml_to_nodes(&xml).unwrap();
        let ad2 = AbbreviatedData::from_xml(&node).unwrap();
        assert!(ad2.is_empty());
    }
}
