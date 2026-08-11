//! 书签动作。
//!
//! 对应 Java: org.ofdrw.core.action.actionType.Bookmark

use super::CTDest;
use super::OfdAction;

/// 书签动作。
///
/// 定义一个书签，关联一个目标位置，对应 GB/T 33190 第 15 章的 Bookmark 动作。
///
/// 对应 Java: org.ofdrw.core.action.actionType.Bookmark
#[derive(Debug, Clone)]
pub struct Bookmark {
    /// 书签名称。
    ///
    /// 对应 Java: Bookmark.name (String)
    pub name: String,

    /// 书签的目标位置。
    ///
    /// 对应 Java: Bookmark.dest (CT_Dest)
    pub dest: CTDest,
}

impl Bookmark {
    /// 创建一个新的书签动作。
    ///
    /// 对应 Java: new Bookmark(String name, CT_Dest dest)
    #[must_use]
    pub fn new(name: impl Into<String>, dest: CTDest) -> Self {
        Self {
            name: name.into(),
            dest,
        }
    }
}

impl OfdAction for Bookmark {
    fn to_xml_string(&self) -> String {
        format!(
            "<ofd:Bookmark Name=\"{}\">{}</ofd:Bookmark>",
            self.name,
            self.dest.to_xml_string()
        )
    }

    fn clone_box(&self) -> Box<dyn OfdAction> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::super::DestType;
    use super::*;

    #[test]
    fn test_bookmark_new() {
        let dest = CTDest::new(1);
        let bm = Bookmark::new("Chapter 1", dest);
        assert_eq!(bm.name, "Chapter 1");
        assert_eq!(bm.dest.page, 1);
    }

    #[test]
    fn test_bookmark_to_xml() {
        let dest = CTDest::new(5).dest_type(DestType::XYZ).left(10.0).top(20.0);
        let bm = Bookmark::new("Section 2", dest);
        let xml = bm.to_xml_string();
        assert!(xml.contains("Name=\"Section 2\""));
        assert!(xml.contains("<ofd:Bookmark"));
        assert!(xml.contains("PageID=\"5\""));
        assert!(xml.contains("Type=\"XYZ\""));
        assert!(xml.contains("</ofd:Bookmark>"));
    }

    #[test]
    fn test_bookmark_clone_debug() {
        let dest = CTDest::new(3);
        let bm = Bookmark::new("test", dest);
        let bm2 = bm.clone();
        assert_eq!(bm2.name, "test");
        let dbg = format!("{bm:?}");
        assert!(dbg.contains("Bookmark"));
    }
}
