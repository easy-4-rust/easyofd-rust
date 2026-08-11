//! 文档内跳转动作。
//!
//! 对应 Java: org.ofdrw.core.action.actionType.Goto

use super::{CTDest, OfdAction};

/// 文档内跳转动作。
///
/// 跳转到当前文档的指定页面和位置，对应 GB/T 33190 第 15 章的 Goto 动作。
///
/// 对应 Java: org.ofdrw.core.action.actionType.Goto
#[derive(Debug, Clone)]
pub struct Goto {
    /// 跳转目标位置。
    ///
    /// 对应 Java: Goto.dest (CT_Dest)
    pub dest: CTDest,
}

impl Goto {
    /// 创建一个新的文档内跳转动作。
    ///
    /// 对应 Java: new Goto(CT_Dest dest)
    #[must_use]
    pub fn new(dest: CTDest) -> Self {
        Self { dest }
    }
}

impl OfdAction for Goto {
    fn to_xml_string(&self) -> String {
        format!("<ofd:Goto>{}</ofd:Goto>", self.dest.to_xml_string())
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
    fn test_goto_new() {
        let dest = CTDest::new(3).dest_type(DestType::XYZ).left(10.0).top(20.0);
        let goto = Goto::new(dest);
        assert_eq!(goto.dest.page, 3);
    }

    #[test]
    fn test_goto_to_xml() {
        let dest = CTDest::new(1).dest_type(DestType::Fit);
        let goto = Goto::new(dest);
        let xml = goto.to_xml_string();
        assert!(xml.contains("<ofd:Goto>"));
        assert!(xml.contains("PageID=\"1\""));
        assert!(xml.contains("Type=\"Fit\""));
        assert!(xml.contains("</ofd:Goto>"));
    }

    #[test]
    fn test_goto_clone_debug() {
        let dest = CTDest::new(5);
        let goto = Goto::new(dest);
        let goto2 = goto.clone();
        assert_eq!(goto2.dest.page, 5);
        let dbg = format!("{goto:?}");
        assert!(dbg.contains("Goto"));
    }
}
