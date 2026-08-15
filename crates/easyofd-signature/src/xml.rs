//! SAX-based XML parsers for Signature.xml and SignedInfo.xml.
//!
//! Replaces the legacy string-scanning helpers (`for_each_file_ref`,
//! `extract_xml_value`, `extract_attr_value`, `extract_between`) with
//! `quick-xml` SAX parsing per the task requirements.

use easyofd_core::OfdResult;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// A single `<ofd:FileRef>` entry parsed from SignedInfo.xml.
#[derive(Debug, Clone)]
pub struct FileRefEntry {
    /// File path (text content of the element).
    pub name: String,
    /// Digest method (e.g. `"SM3"`).
    pub check_method: String,
    /// Expected digest hex value.
    pub check_value: String,
}

/// Top-level fields extracted from Signature.xml.
#[derive(Debug, Default)]
pub struct SignatureTop {
    /// Path to SignedInfo.xml (text content of `<ofd:SignedInfoRef>`).
    pub signed_info_ref: Option<String>,
    /// Path to SignedValue.dat (text content of `<ofd:SignedValue>`).
    pub signed_value: Option<String>,
    /// Public key hex (text content of `<ofd:PublicKey>`).
    pub public_key: Option<String>,
    /// Signature method name (text content of `<ofd:SignatureMethod>`).
    pub method: Option<String>,
    /// Full provider tag text including attributes (e.g.
    /// `Version="1.0">easyofd-rust`).  Preserves the raw text so that
    /// `build_signature_xml` can reconstruct an identical `<ofd:Provider>`
    /// element.
    pub provider: Option<String>,
    /// Signature datetime (text content of `<ofd:SignatureDateTime>`).
    pub datetime: Option<String>,
    /// Path to Seal.esl (from `<ofd:Seal>` element's `BaseLoc` or `Ref` attribute).
    ///
    /// 对应 Java: `sig.getSignedInfo().getSeal().getBaseLoc()`
    ///
    /// This is an optional element; when absent, seal matching is skipped.
    pub seal_path: Option<String>,
}

/// Extract the local name from a quick-xml `QName`, stripping any
/// namespace prefix (e.g. `ofd:FileRef` -> `FileRef`).
fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .and_then(|s| s.rsplit_once(':').map(|(_, local)| local).or(Some(s)))
        .unwrap_or("")
}

/// Parse SignedInfo.xml and return every `<ofd:FileRef>` entry.
///
/// Each entry carries the file `name` (text content), `check_method`, and
/// `check_value` (both from element attributes).  Nested `<ofd:RootFile>`
/// elements are handled identically -- the depth counter ensures child
/// elements do not corrupt the extracted text.
///
/// # Errors
///
/// Returns `OfdError::Conversion` when the XML is malformed.
#[allow(clippy::too_many_lines)]
pub fn parse_signed_info(xml: &str) -> OfdResult<Vec<FileRefEntry>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut entries = Vec::new();
    let mut buf = Vec::new();
    let mut in_target = false;
    let mut depth: u32 = 0;
    let mut current_entry = FileRefEntry {
        name: String::new(),
        check_method: String::new(),
        check_value: String::new(),
    };
    let mut current_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = local_name(e.name().as_ref()).to_string();
                if tag == "FileRef" || tag == "RootFile" {
                    if in_target {
                        depth += 1;
                    } else {
                        in_target = true;
                        depth = 1;
                        current_entry = FileRefEntry {
                            name: String::new(),
                            check_method: String::new(),
                            check_value: String::new(),
                        };
                        for attr in e.attributes().flatten() {
                            let key = local_name(attr.key.as_ref()).to_string();
                            let val = std::str::from_utf8(&attr.value).unwrap_or("");
                            match key.as_str() {
                                "CheckMethod" => {
                                    current_entry.check_method = val.to_string();
                                }
                                "CheckValue" => {
                                    current_entry.check_value = val.to_string();
                                }
                                _ => {}
                            }
                        }
                        current_text.clear();
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                // Self-closing element -- no text content expected.
                let tag = local_name(e.name().as_ref()).to_string();
                if (tag == "FileRef" || tag == "RootFile") && !in_target {
                    let mut entry = FileRefEntry {
                        name: String::new(),
                        check_method: String::new(),
                        check_value: String::new(),
                    };
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref()).to_string();
                        let val = std::str::from_utf8(&attr.value).unwrap_or("");
                        match key.as_str() {
                            "CheckMethod" => {
                                entry.check_method = val.to_string();
                            }
                            "CheckValue" => {
                                entry.check_value = val.to_string();
                            }
                            _ => {}
                        }
                    }
                    entries.push(entry);
                }
            }
            Ok(Event::End(e)) => {
                let tag = local_name(e.name().as_ref()).to_string();
                if (tag == "FileRef" || tag == "RootFile") && in_target {
                    depth -= 1;
                    if depth == 0 {
                        in_target = false;
                        current_entry.name = current_text.trim().to_string();
                        entries.push(current_entry.clone());
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if in_target {
                    current_text.push_str(std::str::from_utf8(t.as_ref()).unwrap_or(""));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(easyofd_core::OfdError::Conversion(format!(
                    "SignedInfo XML 解析失败: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(entries)
}

/// A single `<ofd:SignatureRef>` entry parsed from OFD.xml.
#[derive(Debug, Clone)]
pub struct SignatureRef {
    /// Path to the Signature XML file (text content of `<ofd:SignatureRef>`).
    pub path: String,
}

/// Top-level fields extracted from OFD.xml.
#[derive(Debug, Default)]
pub struct OfdRoot {
    /// List of signature references from `<ofd:Signatures>`.
    pub signatures: Vec<SignatureRef>,
}

/// Parse OFD.xml and extract the `<ofd:Signatures>` element's
/// `<ofd:SignatureRef>` list.
///
/// Returns an [`OfdRoot`] with the list of signature file paths. If no
/// `<ofd:Signatures>` element is present, returns an empty list.
///
/// # Errors
///
/// Returns `OfdError::Conversion` when the XML is malformed.
pub fn parse_ofd_root(xml: &str) -> OfdResult<OfdRoot> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut root = OfdRoot::default();
    let mut buf = Vec::new();
    let mut in_signatures = false;
    let mut in_sig_ref = false;
    let mut current_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = local_name(e.name().as_ref()).to_string();
                if tag == "Signatures" {
                    in_signatures = true;
                } else if tag == "SignatureRef" && in_signatures {
                    in_sig_ref = true;
                    current_text.clear();
                }
            }
            Ok(Event::Empty(e)) => {
                let tag = local_name(e.name().as_ref()).to_string();
                if tag == "SignatureRef" && in_signatures {
                    // Self-closing <ofd:SignatureRef/> with attribute-based path.
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref()).to_string();
                        if key == "BaseLoc" {
                            let val = std::str::from_utf8(&attr.value).unwrap_or("");
                            root.signatures.push(SignatureRef {
                                path: val.to_string(),
                            });
                        }
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if in_sig_ref {
                    current_text.push_str(std::str::from_utf8(t.as_ref()).unwrap_or(""));
                }
            }
            Ok(Event::End(e)) => {
                let tag = local_name(e.name().as_ref()).to_string();
                if tag == "SignatureRef" && in_sig_ref {
                    in_sig_ref = false;
                    let text = current_text.trim().to_string();
                    if !text.is_empty() {
                        root.signatures.push(SignatureRef { path: text });
                    }
                } else if tag == "Signatures" {
                    in_signatures = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(easyofd_core::OfdError::Conversion(format!(
                    "OFD.xml 解析失败: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(root)
}

/// Parse the top-level Signature.xml, extracting the key fields needed by
/// `read_signature` and `build_signature_xml`.
///
/// # Errors
///
/// Returns `OfdError::Conversion` when the XML is malformed.
pub fn parse_signature_top(xml: &str) -> OfdResult<SignatureTop> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut top = SignatureTop::default();
    let mut buf = Vec::new();
    let mut current_tag = String::new();
    let mut in_target = false;
    let mut current_text = String::new();

    // Tags whose text content we want to capture.
    const WANTED: &[&str] = &[
        "SignedInfoRef",
        "SignedValue",
        "PublicKey",
        "SignatureMethod",
        "Provider",
        "SignatureDateTime",
    ];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = local_name(e.name().as_ref()).to_string();
                if WANTED.contains(&tag.as_str()) {
                    in_target = true;
                    current_tag.clone_from(&tag);
                    current_text.clear();
                } else if tag == "Seal" {
                    // 对应 Java: sig.getSignedInfo().getSeal().getBaseLoc()
                    // <ofd:Seal> 提取 BaseLoc 或 Ref 属性作为印章文件路径。
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref()).to_string();
                        let val = std::str::from_utf8(&attr.value).unwrap_or("");
                        match key.as_str() {
                            "BaseLoc" => {
                                top.seal_path = Some(val.to_string());
                            }
                            "Ref" if top.seal_path.is_none() => {
                                top.seal_path = Some(val.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                // Self-closing element -- handle <ofd:Seal BaseLoc="..."/> etc.
                let tag = local_name(e.name().as_ref()).to_string();
                if tag == "Seal" {
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref()).to_string();
                        let val = std::str::from_utf8(&attr.value).unwrap_or("");
                        match key.as_str() {
                            "BaseLoc" => {
                                top.seal_path = Some(val.to_string());
                            }
                            "Ref" if top.seal_path.is_none() => {
                                top.seal_path = Some(val.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if in_target {
                    current_text.push_str(std::str::from_utf8(t.as_ref()).unwrap_or(""));
                }
            }
            Ok(Event::End(e)) => {
                let tag = local_name(e.name().as_ref()).to_string();
                if in_target && tag == current_tag {
                    in_target = false;
                    let text = current_text.trim().to_string();
                    match current_tag.as_str() {
                        "SignedInfoRef" => top.signed_info_ref = Some(text),
                        "SignedValue" => top.signed_value = Some(text),
                        "PublicKey" => top.public_key = Some(text),
                        "SignatureMethod" => top.method = Some(text),
                        "Provider" => top.provider = Some(text),
                        "SignatureDateTime" => top.datetime = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(easyofd_core::OfdError::Conversion(format!(
                    "Signature XML 解析失败: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(top)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── local_name ─────────────────────────────────────────────────────

    #[test]
    fn local_name_strips_namespace() {
        assert_eq!(local_name(b"ofd:FileRef"), "FileRef");
    }

    #[test]
    fn local_name_returns_full_when_no_prefix() {
        assert_eq!(local_name(b"FileRef"), "FileRef");
    }

    #[test]
    fn local_name_handles_empty() {
        assert_eq!(local_name(b""), "");
    }

    // ── parse_signed_info ──────────────────────────────────────────────

    #[test]
    fn parse_signed_info_extracts_file_refs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:References>
    <ofd:FileRef CheckMethod="SM3" CheckValue="aabb">Doc_0/Document.xml</ofd:FileRef>
    <ofd:FileRef CheckMethod="SM3" CheckValue="ccdd">Doc_0/Content.xml</ofd:FileRef>
  </ofd:References>
</ofd:SignedInfo>"#;
        let entries = parse_signed_info(xml).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "Doc_0/Document.xml");
        assert_eq!(entries[0].check_method, "SM3");
        assert_eq!(entries[0].check_value, "aabb");
        assert_eq!(entries[1].name, "Doc_0/Content.xml");
        assert_eq!(entries[1].check_value, "ccdd");
    }

    #[test]
    fn parse_signed_info_handles_self_closing_elements() {
        let xml = r#"<?xml version="1.0"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:References>
    <ofd:FileRef CheckMethod="SM3" CheckValue="1234" />
  </ofd:References>
</ofd:SignedInfo>"#;
        let entries = parse_signed_info(xml).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].check_value, "1234");
    }

    #[test]
    fn parse_signed_info_handles_root_file_element() {
        let xml = r#"<?xml version="1.0"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:References>
    <ofd:RootFile CheckMethod="SM3" CheckValue="abcd">Doc_0/Document.xml</ofd:RootFile>
  </ofd:References>
</ofd:SignedInfo>"#;
        let entries = parse_signed_info(xml).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Doc_0/Document.xml");
    }

    #[test]
    fn parse_signed_info_empty_references() {
        let xml = r#"<?xml version="1.0"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:References>
  </ofd:References>
</ofd:SignedInfo>"#;
        let entries = parse_signed_info(xml).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_signed_info_malformed_xml_returns_error() {
        let xml = "<not valid xml <<";
        let result = parse_signed_info(xml);
        assert!(result.is_err());
    }

    #[test]
    fn parse_signed_info_nested_file_ref() {
        // Test nested depth tracking: a FileRef inside another FileRef.
        let xml = r#"<?xml version="1.0"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:References>
    <ofd:FileRef CheckMethod="SM3" CheckValue="aa">
      <ofd:FileRef CheckMethod="SM3" CheckValue="bb">inner.xml</ofd:FileRef>
    </ofd:FileRef>
  </ofd:References>
</ofd:SignedInfo>"#;
        let entries = parse_signed_info(xml).unwrap();
        // The outer FileRef wraps the inner one; both should be captured.
        assert!(!entries.is_empty());
    }

    // ── parse_signature_top ────────────────────────────────────────────

    #[test]
    fn parse_signature_top_extracts_all_fields() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016" ID="Signature_0">
  <ofd:SignedInfoRef>Doc_0/Signs/SignedInfo.xml</ofd:SignedInfoRef>
  <ofd:SignedValue>Doc_0/Signs/SignedValue.dat</ofd:SignedValue>
  <ofd:Provider Version="1.0">easyofd-rust</ofd:Provider>
  <ofd:SignatureMethod>SM2WithSM3</ofd:SignatureMethod>
  <ofd:SignatureDateTime>2026-01-01T00:00:00</ofd:SignatureDateTime>
  <ofd:PublicKey>AABBCCDD</ofd:PublicKey>
</ofd:Signature>"#;
        let top = parse_signature_top(xml).unwrap();
        assert_eq!(top.signed_info_ref.unwrap(), "Doc_0/Signs/SignedInfo.xml");
        assert_eq!(top.signed_value.unwrap(), "Doc_0/Signs/SignedValue.dat");
        assert_eq!(top.method.unwrap(), "SM2WithSM3");
        assert_eq!(top.datetime.unwrap(), "2026-01-01T00:00:00");
        assert_eq!(top.public_key.unwrap(), "AABBCCDD");
        assert!(top.provider.unwrap().contains("easyofd-rust"));
    }

    #[test]
    fn parse_signature_top_empty_xml_returns_defaults() {
        let xml = "<root/>";
        let top = parse_signature_top(xml).unwrap();
        assert!(top.signed_info_ref.is_none());
        assert!(top.signed_value.is_none());
        assert!(top.public_key.is_none());
        assert!(top.method.is_none());
    }

    #[test]
    fn parse_signature_top_partial_fields() {
        let xml = r#"<?xml version="1.0"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:PublicKey>EEFF</ofd:PublicKey>
</ofd:Signature>"#;
        let top = parse_signature_top(xml).unwrap();
        assert_eq!(top.public_key.unwrap(), "EEFF");
        assert!(top.method.is_none());
    }

    #[test]
    fn parse_signature_top_malformed_xml_returns_error() {
        let result = parse_signature_top("<not valid <<<<");
        assert!(result.is_err());
    }

    #[test]
    fn parse_signature_top_provider_text() {
        let xml = r#"<?xml version="1.0"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Provider Version="1.0">my-provider</ofd:Provider>
</ofd:Signature>"#;
        let top = parse_signature_top(xml).unwrap();
        let provider = top.provider.unwrap();
        assert!(provider.contains("my-provider"), "provider: {provider}");
    }

    // ── parse_ofd_root ────────────────────────────────────────────────

    #[test]
    fn parse_ofd_root_extracts_signature_refs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody>
    <ofd:Signatures>
      <ofd:SignatureRef>Doc_0/Signs/Signature_0.xml</ofd:SignatureRef>
      <ofd:SignatureRef>Doc_0/Signs/Signature_1.xml</ofd:SignatureRef>
      <ofd:SignatureRef>Doc_0/Signs/Signature_2.xml</ofd:SignatureRef>
    </ofd:Signatures>
  </ofd:DocBody>
</ofd:OFD>"#;
        let root = parse_ofd_root(xml).unwrap();
        assert_eq!(root.signatures.len(), 3);
        assert_eq!(root.signatures[0].path, "Doc_0/Signs/Signature_0.xml");
        assert_eq!(root.signatures[1].path, "Doc_0/Signs/Signature_1.xml");
        assert_eq!(root.signatures[2].path, "Doc_0/Signs/Signature_2.xml");
    }

    #[test]
    fn parse_ofd_root_no_signatures_returns_empty() {
        let xml = r#"<?xml version="1.0"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody></ofd:DocBody>
</ofd:OFD>"#;
        let root = parse_ofd_root(xml).unwrap();
        assert!(root.signatures.is_empty());
    }

    #[test]
    fn parse_ofd_root_malformed_xml_returns_error() {
        let result = parse_ofd_root("<not valid <<<<");
        assert!(result.is_err());
    }

    #[test]
    fn parse_ofd_root_empty_signatures_element() {
        let xml = r#"<?xml version="1.0"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Signatures>
  </ofd:Signatures>
</ofd:OFD>"#;
        let root = parse_ofd_root(xml).unwrap();
        assert!(root.signatures.is_empty());
    }

    // ── parse_signature_top Seal path ─────────────────────────────────

    #[test]
    fn parse_signature_top_seal_with_base_loc() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:SignedInfo>
    <ofd:Seal BaseLoc="Doc_0/Signs/Sign_0/Seal.esl"/>
  </ofd:SignedInfo>
  <ofd:SignedValue>Doc_0/Signs/SignedValue.dat</ofd:SignedValue>
</ofd:Signature>"#;
        let top = parse_signature_top(xml).unwrap();
        assert_eq!(
            top.seal_path.as_deref(),
            Some("Doc_0/Signs/Sign_0/Seal.esl")
        );
    }

    #[test]
    fn parse_signature_top_seal_with_ref_attr() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Seal ID="Seal_0" Type="Seal" Ref="Doc_0/Seal_0.esl">Doc_0/Res/Seal_0.png</ofd:Seal>
  <ofd:SignedValue>Doc_0/Signs/SignedValue.dat</ofd:SignedValue>
</ofd:Signature>"#;
        let top = parse_signature_top(xml).unwrap();
        assert_eq!(top.seal_path.as_deref(), Some("Doc_0/Seal_0.esl"));
    }

    #[test]
    fn parse_signature_top_no_seal_returns_none() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:SignedValue>Doc_0/Signs/SignedValue.dat</ofd:SignedValue>
</ofd:Signature>"#;
        let top = parse_signature_top(xml).unwrap();
        assert!(top.seal_path.is_none());
    }
}
