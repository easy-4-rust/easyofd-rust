//! 附件打开动作。
//!
//! 对应 Java: org.ofdrw.core.action.actionType.GotoA

use super::OfdAction;

/// 附件打开动作。
///
/// 打开一个已附加到 OFD 文档的附件文件，对应 GB/T 33190 第 15 章的 GotoA 动作。
///
/// 对应 Java: org.ofdrw.core.action.actionType.GotoA
#[derive(Debug, Clone)]
pub struct GotoA {
    /// 附件的标识 ID。
    ///
    /// 对应 Java: GotoA.attachID (String)
    pub attach_id: String,
}

impl GotoA {
    /// 创建一个新的附件打开动作。
    ///
    /// 对应 Java: new GotoA(String attachID)
    #[must_use]
    pub fn new(attach_id: impl Into<String>) -> Self {
        Self {
            attach_id: attach_id.into(),
        }
    }
}

impl OfdAction for GotoA {
    fn to_xml_string(&self) -> String {
        format!("<ofd:GotoA AttachID=\"{}\"/>", self.attach_id)
    }

    fn clone_box(&self) -> Box<dyn OfdAction> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gotoa_new() {
        let gotoa = GotoA::new("att_001");
        assert_eq!(gotoa.attach_id, "att_001");
    }

    #[test]
    fn test_gotoa_to_xml() {
        let gotoa = GotoA::new("att_002");
        let xml = gotoa.to_xml_string();
        assert!(xml.contains("AttachID=\"att_002\""));
        assert!(xml.contains("<ofd:GotoA"));
        assert!(xml.ends_with("/>"));
    }

    #[test]
    fn test_gotoa_from_string() {
        let s = String::from("attachment_1");
        let gotoa = GotoA::new(s);
        assert_eq!(gotoa.attach_id, "attachment_1");
    }

    #[test]
    fn test_gotoa_clone_debug() {
        let gotoa = GotoA::new("att_1");
        let gotoa2 = gotoa.clone();
        assert_eq!(gotoa2.attach_id, "att_1");
        let dbg = format!("{gotoa:?}");
        assert!(dbg.contains("GotoA"));
    }
}
