//! URI 超链接动作。
//!
//! 对应 Java: org.ofdrw.core.action.actionType.URI

use super::OfdAction;

/// URI 超链接动作。
///
/// 打开一个 URI（统一资源标识符），对应 GB/T 33190 第 15 章的 URI 动作。
///
/// 对应 Java: org.ofdrw.core.action.actionType.URI
#[derive(Debug, Clone)]
pub struct URI {
    /// 超链接的 URI 地址。
    ///
    /// 对应 Java: URI.uri (String)
    pub uri: String,
}

impl URI {
    /// 创建一个新的 URI 动作。
    ///
    /// 对应 Java: new URI(String uri)
    #[must_use]
    pub fn new(uri: impl Into<String>) -> Self {
        Self { uri: uri.into() }
    }
}

impl OfdAction for URI {
    fn to_xml_string(&self) -> String {
        format!("<ofd:URI URI=\"{}\"/>", self.uri)
    }

    fn clone_box(&self) -> Box<dyn OfdAction> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_new() {
        let uri = URI::new("https://example.com");
        assert_eq!(uri.uri, "https://example.com");
    }

    #[test]
    fn test_uri_to_xml() {
        let uri = URI::new("https://example.com/path?q=1");
        let xml = uri.to_xml_string();
        assert!(xml.contains("URI=\"https://example.com/path?q=1\""));
        assert!(xml.contains("<ofd:URI"));
        assert!(xml.ends_with("/>"));
    }

    #[test]
    fn test_uri_from_string() {
        let s = String::from("http://test.org");
        let uri = URI::new(s);
        assert_eq!(uri.uri, "http://test.org");
    }

    #[test]
    fn test_uri_clone_debug() {
        let uri = URI::new("https://a.b");
        let uri2 = uri.clone();
        assert_eq!(uri2.uri, "https://a.b");
        let dbg = format!("{uri:?}");
        assert!(dbg.contains("URI"));
    }
}
