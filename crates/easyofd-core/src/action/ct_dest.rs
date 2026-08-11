//! 目标位置定义。
//!
//! 对应 Java: org.ofdrw.core.action.CT_Dest

use std::fmt::Write;

use super::DestType;

/// 目标位置。
///
/// 定义页面跳转的目标位置参数，包括目标页码、显示类型和坐标。
///
/// 对应 Java: org.ofdrw.core.action.CT_Dest
#[derive(Debug, Clone, PartialEq)]
pub struct CTDest {
    /// 目标页码。
    ///
    /// 对应 Java: CT_Dest.page (int)
    pub page: u32,

    /// 目标显示类型。
    ///
    /// 对应 Java: CT_Dest.type (DestType)
    pub dest_type: DestType,

    /// 左上角 X 坐标（单位 mm）。
    ///
    /// 仅在 `dest_type` 为 `XYZ`、`FitH`、`FitBH` 时有效。
    /// 对应 Java: CT_Dest.left (Double)
    pub left: Option<f64>,

    /// 左上角 Y 坐标（单位 mm）。
    ///
    /// 仅在 `dest_type` 为 `XYZ`、`FitV`、`FitBV` 时有效。
    /// 对应 Java: CT_Dest.top (Double)
    pub top: Option<f64>,

    /// 缩放级别。
    ///
    /// 仅在 `dest_type` 为 `XYZ` 时有效。
    /// 对应 Java: CT_Dest.zoom (Double)
    pub zoom: Option<f64>,
}

impl CTDest {
    /// 创建一个新的目标位置。
    ///
    /// 对应 Java: new CT_Dest(int page)
    #[must_use]
    pub fn new(page: u32) -> Self {
        Self {
            page,
            dest_type: DestType::Fit,
            left: None,
            top: None,
            zoom: None,
        }
    }

    /// 设置目标显示类型。
    ///
    /// 对应 Java: CT_Dest.setType(DestType)
    #[must_use]
    pub fn dest_type(mut self, dest_type: DestType) -> Self {
        self.dest_type = dest_type;
        self
    }

    /// 设置左上角 X 坐标。
    ///
    /// 对应 Java: CT_Dest.setLeft(Double)
    #[must_use]
    pub fn left(mut self, left: f64) -> Self {
        self.left = Some(left);
        self
    }

    /// 设置左上角 Y 坐标。
    ///
    /// 对应 Java: CT_Dest.setTop(Double)
    #[must_use]
    pub fn top(mut self, top: f64) -> Self {
        self.top = Some(top);
        self
    }

    /// 设置缩放级别。
    ///
    /// 对应 Java: CT_Dest.setZoom(Double)
    #[must_use]
    pub fn zoom(mut self, zoom: f64) -> Self {
        self.zoom = Some(zoom);
        self
    }

    /// 序列化为 OFD XML 字符串。
    ///
    /// 输出标准 OFD XML 格式。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut attrs = format!("PageID=\"{}\" Type=\"{}\"", self.page, self.dest_type);

        if let Some(left) = self.left {
            let _ = write!(attrs, " Left=\"{left}\"");
        }
        if let Some(top) = self.top {
            let _ = write!(attrs, " Top=\"{top}\"");
        }
        if let Some(zoom) = self.zoom {
            let _ = write!(attrs, " Zoom=\"{zoom}\"");
        }

        format!("<ofd:Dest {attrs}/>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_dest_new() {
        let dest = CTDest::new(3);
        assert_eq!(dest.page, 3);
        assert_eq!(dest.dest_type, DestType::Fit);
        assert!(dest.left.is_none());
        assert!(dest.top.is_none());
        assert!(dest.zoom.is_none());
    }

    #[test]
    fn test_ct_dest_builder() {
        let dest = CTDest::new(1)
            .dest_type(DestType::XYZ)
            .left(10.0)
            .top(20.0)
            .zoom(1.5);
        assert_eq!(dest.page, 1);
        assert_eq!(dest.dest_type, DestType::XYZ);
        assert_eq!(dest.left, Some(10.0));
        assert_eq!(dest.top, Some(20.0));
        assert_eq!(dest.zoom, Some(1.5));
    }

    #[test]
    fn test_ct_dest_to_xml_fit() {
        let dest = CTDest::new(5);
        let xml = dest.to_xml_string();
        assert!(xml.contains("PageID=\"5\""));
        assert!(xml.contains("Type=\"Fit\""));
        assert!(!xml.contains("Left"));
    }

    #[test]
    fn test_ct_dest_to_xml_xyz() {
        let dest = CTDest::new(2)
            .dest_type(DestType::XYZ)
            .left(10.5)
            .top(20.5)
            .zoom(2.0);
        let xml = dest.to_xml_string();
        assert!(xml.contains("PageID=\"2\""));
        assert!(xml.contains("Type=\"XYZ\""));
        assert!(xml.contains("Left=\"10.5\""));
        assert!(xml.contains("Top=\"20.5\""));
        assert!(xml.contains("Zoom=\"2\""));
    }

    #[test]
    fn test_ct_dest_clone_debug() {
        let dest = CTDest::new(1).dest_type(DestType::FitH).left(5.0);
        let dest2 = dest.clone();
        assert_eq!(dest2.page, 1);
        assert_eq!(dest2.dest_type, DestType::FitH);
        let dbg = format!("{dest:?}");
        assert!(dbg.contains("CTDest"));
    }
}
