//! OFD 合规规则引擎。
//!
//! 对应 Java: `org.ofdrw.archive.check.rule` 包。
//!
//! 提供 [`ComplianceRule`] trait 及五条基础合规规则，用于校验 OFD 包
//! 是否符合 GB/T 33190-2016 / GMT 0099 规范。

use std::collections::HashSet;
use std::io::Cursor;

use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

// ─── 扩展规则模块 ────────────────────────────────────────────────────────────

/// 字体合规规则。
pub mod font_rule;
/// 图片合规规则。
pub mod image_rule;
/// 路径合规规则。
pub mod path_rule;
/// 签名合规规则。
pub mod signature_rule;
/// 文本合规规则。
pub mod text_rule;

// ─── 公共类型 ────────────────────────────────────────────────────────────────

/// 合规规则接口。
///
/// 对应 Java: `org.ofdrw.archive.check.rule.ComplianceRule`
pub trait ComplianceRule: Send + Sync {
    /// 规则名称。
    fn name(&self) -> &'static str;

    /// 执行检查。
    ///
    /// `entries` 是 OFD 包内所有文件的 `(路径, 内容)` 列表。
    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult;
}

/// 单条规则的检查结果。
///
/// 对应 Java: `org.ofdrw.archive.check.rule.RuleResult`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleResult {
    /// 是否通过。
    pub passed: bool,
    /// 结果说明信息。
    pub message: String,
}

// ─── 工具函数 ────────────────────────────────────────────────────────────────

/// 从条目列表中查找指定路径的内容。
fn find_entry<'a>(entries: &'a [(String, Vec<u8>)], path: &str) -> Option<&'a [u8]> {
    entries
        .iter()
        .find(|(name, _)| name == path)
        .map(|(_, data)| data.as_slice())
}

/// 读取 XML 文本内容（简化版：跳过标签，收集 Text 事件）。
#[allow(dead_code)]
fn xml_text_content(xml_bytes: &[u8]) -> String {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                if let Ok(s) = t.xml10_content() {
                    text.push_str(&s);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    text
}

/// 从 OFD.xml 中提取指定属性值。
fn ofd_xml_attr(xml_bytes: &[u8], attr_name: &[u8]) -> Option<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"ofd:OFD" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == attr_name {
                        return attr
                            .decoded_and_normalized_value(
                                quick_xml::XmlVersion::Explicit1_0,
                                reader.decoder(),
                            )
                            .ok()
                            .map(|v| v.to_string());
                    }
                }
                return None;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// 从 OFD.xml 中提取 `DocRoot` 元素的文本内容。
fn ofd_xml_doc_root(xml_bytes: &[u8]) -> Option<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_doc_root = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"ofd:DocRoot" => {
                in_doc_root = true;
            }
            Ok(Event::Text(ref t)) if in_doc_root => {
                return t.xml10_content().ok().map(|c| c.into_owned());
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"ofd:DocRoot" => {
                in_doc_root = false;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// 从 Document.xml 中提取所有 `BaseLoc` 属性。
fn document_page_locs(xml_bytes: &[u8]) -> Vec<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut locs = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) if e.name().as_ref() == b"ofd:Page" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"BaseLoc" {
                        if let Ok(val) = attr.decoded_and_normalized_value(
                            quick_xml::XmlVersion::Explicit1_0,
                            reader.decoder(),
                        ) {
                            locs.push(val.to_string());
                        }
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    locs
}

/// 从 Document.xml 中提取所有引用路径（PublicRes、DocumentRes、媒体文件等）。
#[allow(dead_code)]
fn document_resource_refs(xml_bytes: &[u8]) -> Vec<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut refs = Vec::new();
    let mut in_target = false;
    let targets: &[&[u8]] = &[
        b"ofd:PublicRes",
        b"ofd:DocumentRes",
        b"ofd:Attachment",
        b"ofd:Annotation",
    ];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if targets.contains(&e.name().as_ref()) {
                    in_target = true;
                }
            }
            Ok(Event::Text(ref t)) if in_target => {
                if let Ok(s) = t.xml10_content() {
                    let trimmed = s.trim().to_string();
                    if !trimmed.is_empty() {
                        refs.push(trimmed);
                    }
                }
            }
            Ok(Event::End(ref e)) if targets.contains(&e.name().as_ref()) => {
                in_target = false;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    refs
}

/// XML 命名空间 URI 前缀白名单 —— 这些是规范定义的命名空间，不算外部引用。
const NAMESPACE_PREFIXES: &[&str] = &[
    "http://www.ofdspec.org/2016",
    "http://www.w3.org/",
    "http://xml.org/",
];

/// 检查 XML 内容是否包含外部资源引用（http/https/ftp 等绝对 URL）。
///
/// XML 命名空间声明（`xmlns:*`）和已知规范命名空间 URI 会被跳过。
fn contains_external_refs(xml_bytes: &[u8]) -> bool {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                if let Ok(s) = t.xml10_content() {
                    if contains_external_url(&s) {
                        return true;
                    }
                }
            }
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                for attr in e.attributes().flatten() {
                    let key = attr.key.as_ref();
                    // 跳过 xmlns 命名空间声明
                    if key == b"xmlns" || key.starts_with(b"xmlns:") {
                        continue;
                    }
                    if let Ok(val) = attr.decoded_and_normalized_value(
                        quick_xml::XmlVersion::Explicit1_0,
                        reader.decoder(),
                    ) {
                        if contains_external_url(&val) {
                            return true;
                        }
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    false
}

/// 检查文本是否包含外部 URL（排除已知命名空间 URI）。
fn contains_external_url(text: &str) -> bool {
    let lower = text.to_lowercase();
    if !lower.contains("http://")
        && !lower.contains("https://")
        && !lower.contains("ftp://")
        && !lower.contains("file://")
    {
        return false;
    }
    // 排除已知命名空间 URI
    for prefix in NAMESPACE_PREFIXES {
        if text.starts_with(prefix) {
            return false;
        }
    }
    true
}

// ─── Rule 1: DocTypeRule ─────────────────────────────────────────────────────

/// 对应 Java: `org.ofdrw.archive.check.rule.DocTypeRule`
///
/// 校验 `OFD.xml` 根元素必须包含 `DocType="OFD"` 属性。
#[derive(Debug, Clone, Copy)]
pub struct DocTypeRule;

impl ComplianceRule for DocTypeRule {
    fn name(&self) -> &'static str {
        "DocTypeRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let ofd_xml = match find_entry(entries, "OFD.xml") {
            Some(data) => data,
            None => {
                return RuleResult {
                    passed: false,
                    message: "OFD.xml 不存在".into(),
                };
            }
        };

        match ofd_xml_attr(ofd_xml, b"DocType") {
            Some(ref doctype) if doctype == "OFD" => RuleResult {
                passed: true,
                message: "DocType=\"OFD\" 校验通过".into(),
            },
            Some(other) => RuleResult {
                passed: false,
                message: format!("DocType 属性值应为 \"OFD\"，实际为 \"{other}\""),
            },
            None => RuleResult {
                passed: false,
                message: "OFD.xml 根元素缺少 DocType 属性".into(),
            },
        }
    }
}

// ─── Rule 2: VersionRule ─────────────────────────────────────────────────────

/// 对应 Java: `org.ofdrw.archive.check.rule.VersionRule`
///
/// 校验 `OFD.xml` 根元素的 `Version` 属性必须为 `"1.2"`。
#[derive(Debug, Clone, Copy)]
pub struct VersionRule;

impl ComplianceRule for VersionRule {
    fn name(&self) -> &'static str {
        "VersionRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let ofd_xml = match find_entry(entries, "OFD.xml") {
            Some(data) => data,
            None => {
                return RuleResult {
                    passed: false,
                    message: "OFD.xml 不存在".into(),
                };
            }
        };

        match ofd_xml_attr(ofd_xml, b"Version") {
            Some(ref version) if version == "1.2" => RuleResult {
                passed: true,
                message: "Version=\"1.2\" 校验通过".into(),
            },
            Some(other) => RuleResult {
                passed: false,
                message: format!("Version 应为 \"1.2\"，实际为 \"{other}\""),
            },
            None => RuleResult {
                passed: false,
                message: "OFD.xml 根元素缺少 Version 属性".into(),
            },
        }
    }
}

// ─── Rule 3: DocRootRule ─────────────────────────────────────────────────────

/// 对应 Java: `org.ofdrw.archive.check.rule.DocRootRule`
///
/// 校验 `OFD.xml` 中 `DocRoot` 必须指向 `Doc_0/Document.xml`。
#[derive(Debug, Clone, Copy)]
pub struct DocRootRule;

impl ComplianceRule for DocRootRule {
    fn name(&self) -> &'static str {
        "DocRootRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let ofd_xml = match find_entry(entries, "OFD.xml") {
            Some(data) => data,
            None => {
                return RuleResult {
                    passed: false,
                    message: "OFD.xml 不存在".into(),
                };
            }
        };

        match ofd_xml_doc_root(ofd_xml) {
            Some(ref root) if root == "Doc_0/Document.xml" => RuleResult {
                passed: true,
                message: "DocRoot 指向 Doc_0/Document.xml 校验通过".into(),
            },
            Some(other) => RuleResult {
                passed: false,
                message: format!("DocRoot 应为 \"Doc_0/Document.xml\"，实际为 \"{other}\""),
            },
            None => RuleResult {
                passed: false,
                message: "OFD.xml 中缺少 DocRoot 元素".into(),
            },
        }
    }
}

// ─── Rule 4: PagesExistRule ──────────────────────────────────────────────────

/// 对应 Java: `org.ofdrw.archive.check.rule.PagesExistRule`
///
/// 校验 `Document.xml` 中声明的每个 Page 的 `BaseLoc` 对应的文件
/// 必须存在于 OFD 包中。
#[derive(Debug, Clone, Copy)]
pub struct PagesExistRule;

impl ComplianceRule for PagesExistRule {
    fn name(&self) -> &'static str {
        "PagesExistRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let doc_xml = match find_entry(entries, "Doc_0/Document.xml") {
            Some(data) => data,
            None => {
                return RuleResult {
                    passed: false,
                    message: "Doc_0/Document.xml 不存在".into(),
                };
            }
        };

        let page_locs = document_page_locs(doc_xml);
        if page_locs.is_empty() {
            return RuleResult {
                passed: false,
                message: "Document.xml 中未声明任何 Page".into(),
            };
        }

        let entry_names: HashSet<&str> = entries.iter().map(|(name, _)| name.as_str()).collect();
        let mut missing = Vec::new();

        for loc in &page_locs {
            let full_path = format!("Doc_0/{loc}");
            if !entry_names.contains(full_path.as_str()) {
                missing.push(loc.clone());
            }
        }

        if missing.is_empty() {
            RuleResult {
                passed: true,
                message: format!("全部 {} 个页面文件存在", page_locs.len()),
            }
        } else {
            RuleResult {
                passed: false,
                message: format!("以下页面文件缺失: {}", missing.join(", ")),
            }
        }
    }
}

// ─── Rule 5: NoExternalResourceRule ──────────────────────────────────────────

/// 对应 Java: `org.ofdrw.archive.check.rule.NoExternalResourceRule`
///
/// 校验 OFD 包内所有 XML 文件不允许引用外部资源（http/https/ftp/file URL）。
#[derive(Debug, Clone, Copy)]
pub struct NoExternalResourceRule;

impl ComplianceRule for NoExternalResourceRule {
    fn name(&self) -> &'static str {
        "NoExternalResourceRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let mut violating_files = Vec::new();

        for (name, data) in entries {
            if std::path::Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
                && contains_external_refs(data)
            {
                violating_files.push(name.clone());
            }
        }

        if violating_files.is_empty() {
            RuleResult {
                passed: true,
                message: "未发现外部资源引用".into(),
            }
        } else {
            RuleResult {
                passed: false,
                message: format!("以下文件包含外部资源引用: {}", violating_files.join(", ")),
            }
        }
    }
}

// ─── 测试 ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 构建条目列表辅助函数。
    fn make_entries(pairs: &[(&str, &[u8])]) -> Vec<(String, Vec<u8>)> {
        pairs
            .iter()
            .map(|(name, data)| (name.to_string(), data.to_vec()))
            .collect()
    }

    // ── DocTypeRule ────────────────────────────────────────────────────────

    #[test]
    fn doc_type_rule_passes() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" DocType="OFD" Version="1.2">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = DocTypeRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn doc_type_rule_fails_missing() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.2">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = DocTypeRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("缺少 DocType"));
    }

    #[test]
    fn doc_type_rule_fails_wrong_value() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" DocType="PDF" Version="1.2">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = DocTypeRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("PDF"));
    }

    #[test]
    fn doc_type_rule_fails_no_ofd_xml() {
        let entries = make_entries(&[]);
        let result = DocTypeRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("不存在"));
    }

    // ── VersionRule ────────────────────────────────────────────────────────

    #[test]
    fn version_rule_passes() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.2">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = VersionRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn version_rule_fails_wrong_version() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.0">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = VersionRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("1.0"));
    }

    #[test]
    fn version_rule_fails_missing() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = VersionRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("缺少 Version"));
    }

    // ── DocRootRule ────────────────────────────────────────────────────────

    #[test]
    fn doc_root_rule_passes() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = DocRootRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn doc_root_rule_fails_wrong_path() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody><ofd:DocRoot>Doc_1/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = DocRootRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("Doc_1"));
    }

    #[test]
    fn doc_root_rule_fails_missing_element() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody><ofd:DocInfo/></ofd:DocBody>
</ofd:OFD>"#;
        let entries = make_entries(&[("OFD.xml", xml)]);
        let result = DocRootRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("缺少 DocRoot"));
    }

    // ── PagesExistRule ─────────────────────────────────────────────────────

    #[test]
    fn pages_exist_rule_passes() {
        let doc_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Pages>
    <ofd:Page ID="1" BaseLoc="Pages/Page_0.xml"/>
    <ofd:Page ID="2" BaseLoc="Pages/Page_1.xml"/>
  </ofd:Pages>
</ofd:Document>"#;
        let page0 = b"<ofd:Page/>";
        let page1 = b"<ofd:Page/>";
        let entries = make_entries(&[
            ("Doc_0/Document.xml", doc_xml),
            ("Doc_0/Pages/Page_0.xml", page0),
            ("Doc_0/Pages/Page_1.xml", page1),
        ]);
        let result = PagesExistRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn pages_exist_rule_fails_missing_page() {
        let doc_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Pages>
    <ofd:Page ID="1" BaseLoc="Pages/Page_0.xml"/>
    <ofd:Page ID="2" BaseLoc="Pages/Page_1.xml"/>
  </ofd:Pages>
</ofd:Document>"#;
        let page0 = b"<ofd:Page/>";
        let entries = make_entries(&[
            ("Doc_0/Document.xml", doc_xml),
            ("Doc_0/Pages/Page_0.xml", page0),
        ]);
        let result = PagesExistRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("Page_1.xml"));
    }

    #[test]
    fn pages_exist_rule_fails_no_document_xml() {
        let entries = make_entries(&[]);
        let result = PagesExistRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("Document.xml 不存在"));
    }

    #[test]
    fn pages_exist_rule_fails_no_pages() {
        let doc_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Pages/>
</ofd:Document>"#;
        let entries = make_entries(&[("Doc_0/Document.xml", doc_xml)]);
        let result = PagesExistRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("未声明任何 Page"));
    }

    // ── NoExternalResourceRule ─────────────────────────────────────────────

    #[test]
    fn no_external_resource_rule_passes() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Pages><ofd:Page ID="1" BaseLoc="Pages/Page_0.xml"/></ofd:Pages>
</ofd:Document>"#;
        let entries = make_entries(&[("Doc_0/Document.xml", xml)]);
        let result = NoExternalResourceRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn no_external_resource_rule_fails_http() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:TextCode>http://evil.com/steal</ofd:TextCode>
</ofd:Document>"#;
        let entries = make_entries(&[("Doc_0/Document.xml", xml)]);
        let result = NoExternalResourceRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("外部资源"));
    }

    #[test]
    fn no_external_resource_rule_fails_https() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:TextCode>https://example.com/resource</ofd:TextCode>
</ofd:Document>"#;
        let entries = make_entries(&[("Doc_0/Document.xml", xml)]);
        let result = NoExternalResourceRule.check(&entries);
        assert!(!result.passed);
    }

    #[test]
    fn no_external_resource_rule_passes_for_non_xml() {
        // 非 XML 文件不检查
        let data = b"http://example.com/image.png";
        let entries = make_entries(&[("Doc_0/Res/image.txt", data)]);
        let result = NoExternalResourceRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    // ── trait 元信息 ──────────────────────────────────────────────────────

    #[test]
    fn rule_names_are_distinct() {
        let rules: Vec<Box<dyn ComplianceRule>> = vec![
            Box::new(DocTypeRule),
            Box::new(VersionRule),
            Box::new(DocRootRule),
            Box::new(PagesExistRule),
            Box::new(NoExternalResourceRule),
        ];
        let names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
        let unique: HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len(), "rule names must be unique");
    }

    #[test]
    fn rule_result_debug_clone_eq() {
        let r = RuleResult {
            passed: true,
            message: "ok".into(),
        };
        let cloned = r.clone();
        assert!(format!("{r:?}").contains("RuleResult"));
        assert_eq!(r, cloned);
    }
}
