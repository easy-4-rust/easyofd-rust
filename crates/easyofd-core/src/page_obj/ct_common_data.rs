//! CT_CommonData 文档公共数据。

use super::{CT_PageArea, CT_TemplatePage};

/// 对应 Java: org.ofdrw.core.basicStructure.doc.CT_CommonData
///
/// 文档公共数据结构，包含页面区域、资源引用、模板页等全局配置。
/// 对应 GB/T 33190-2016 图 6。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_CommonData {
    /// 最大对象 ID。
    pub max_unit_id: Option<u32>,
    /// 页面区域。
    pub page_area: Option<CT_PageArea>,
    /// 公共资源引用路径列表。
    pub public_res: Vec<String>,
    /// 文档资源引用路径列表。
    pub document_res: Vec<String>,
    /// 模板页列表。
    pub template_pages: Vec<CT_TemplatePage>,
    /// 默认颜色空间 ID。
    pub default_cs: Option<u32>,
}

impl CT_CommonData {
    /// 创建空的文档公共数据。
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_unit_id: None,
            page_area: None,
            public_res: Vec::new(),
            document_res: Vec::new(),
            template_pages: Vec::new(),
            default_cs: None,
        }
    }

    /// 设置最大对象 ID。
    #[must_use]
    pub fn max_unit_id(mut self, id: u32) -> Self {
        self.max_unit_id = Some(id);
        self
    }

    /// 设置页面区域。
    #[must_use]
    pub fn page_area(mut self, page_area: CT_PageArea) -> Self {
        self.page_area = Some(page_area);
        self
    }

    /// 添加公共资源引用。
    pub fn add_public_res(&mut self, res: impl Into<String>) {
        self.public_res.push(res.into());
    }

    /// 设置公共资源引用（替换所有）。
    #[must_use]
    pub fn public_res(mut self, res: impl Into<String>) -> Self {
        self.public_res = vec![res.into()];
        self
    }

    /// 添加文档资源引用。
    pub fn add_document_res(&mut self, res: impl Into<String>) {
        self.document_res.push(res.into());
    }

    /// 设置文档资源引用（替换所有）。
    #[must_use]
    pub fn document_res(mut self, res: impl Into<String>) -> Self {
        self.document_res = vec![res.into()];
        self
    }

    /// 添加模板页。
    pub fn add_template_page(&mut self, tpl: CT_TemplatePage) {
        self.template_pages.push(tpl);
    }

    /// 设置默认颜色空间 ID。
    #[must_use]
    pub fn default_cs(mut self, id: u32) -> Self {
        self.default_cs = Some(id);
        self
    }

    /// 获取最大对象 ID。
    #[must_use]
    pub fn get_max_unit_id(&self) -> Option<u32> {
        self.max_unit_id
    }

    /// 获取页面区域。
    #[must_use]
    pub fn get_page_area(&self) -> Option<&CT_PageArea> {
        self.page_area.as_ref()
    }

    /// 获取公共资源列表。
    #[must_use]
    pub fn get_public_res(&self) -> &[String] {
        &self.public_res
    }

    /// 获取文档资源列表。
    #[must_use]
    pub fn get_document_res(&self) -> &[String] {
        &self.document_res
    }

    /// 获取模板页列表。
    #[must_use]
    pub fn get_template_pages(&self) -> &[CT_TemplatePage] {
        &self.template_pages
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = String::from("<ofd:CommonData>\n");
        if let Some(id) = self.max_unit_id {
            writeln!(xml, "  <ofd:MaxUnitID>{id}</ofd:MaxUnitID>").unwrap();
        }
        if let Some(ref pa) = self.page_area {
            writeln!(xml, "  {}", pa.to_xml_string()).unwrap();
        }
        for res in &self.public_res {
            writeln!(xml, "  <ofd:PublicRes>{res}</ofd:PublicRes>").unwrap();
        }
        for res in &self.document_res {
            writeln!(xml, "  <ofd:DocumentRes>{res}</ofd:DocumentRes>").unwrap();
        }
        for tpl in &self.template_pages {
            writeln!(xml, "  {}", tpl.to_xml_string()).unwrap();
        }
        if let Some(cs) = self.default_cs {
            writeln!(xml, "  <ofd:DefaultCS>{cs}</ofd:DefaultCS>").unwrap();
        }
        xml.push_str("</ofd:CommonData>\n");
        xml
    }
}

impl Default for CT_CommonData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_common_data_new() {
        let cd = CT_CommonData::new();
        assert!(cd.max_unit_id.is_none());
        assert!(cd.page_area.is_none());
        assert!(cd.public_res.is_empty());
        assert!(cd.document_res.is_empty());
        assert!(cd.template_pages.is_empty());
        assert!(cd.default_cs.is_none());
    }

    #[test]
    fn test_ct_common_data_builder() {
        let cd = CT_CommonData::new()
            .max_unit_id(100)
            .page_area(CT_PageArea::with_physical(0.0, 0.0, 210.0, 297.0))
            .public_res("PublicRes.xml")
            .document_res("DocumentRes.xml")
            .default_cs(1);
        assert_eq!(cd.get_max_unit_id(), Some(100));
        assert!(cd.get_page_area().is_some());
        assert_eq!(cd.get_public_res(), &["PublicRes.xml"]);
        assert_eq!(cd.get_document_res(), &["DocumentRes.xml"]);
    }

    #[test]
    fn test_ct_common_data_add_resources() {
        let mut cd = CT_CommonData::new();
        cd.add_public_res("res1.xml");
        cd.add_public_res("res2.xml");
        cd.add_document_res("doc_res.xml");
        assert_eq!(cd.get_public_res().len(), 2);
        assert_eq!(cd.get_document_res().len(), 1);
    }

    #[test]
    fn test_ct_common_data_add_template_page() {
        let mut cd = CT_CommonData::new();
        cd.add_template_page(CT_TemplatePage::new(1).name("tpl1"));
        cd.add_template_page(CT_TemplatePage::new(2).name("tpl2"));
        assert_eq!(cd.get_template_pages().len(), 2);
    }

    #[test]
    fn test_ct_common_data_to_xml_basic() {
        let cd = CT_CommonData::new();
        let xml = cd.to_xml_string();
        assert!(xml.contains("<ofd:CommonData>"));
        assert!(xml.contains("</ofd:CommonData>"));
    }

    #[test]
    fn test_ct_common_data_to_xml_full() {
        let mut cd = CT_CommonData::new()
            .max_unit_id(50)
            .page_area(CT_PageArea::with_physical(0.0, 0.0, 210.0, 297.0))
            .default_cs(2);
        cd.add_public_res("PublicRes.xml");
        cd.add_template_page(CT_TemplatePage::new(1).name("bg"));
        let xml = cd.to_xml_string();
        assert!(xml.contains("<ofd:MaxUnitID>50</ofd:MaxUnitID>"));
        assert!(xml.contains("ofd:PageArea"));
        assert!(xml.contains("<ofd:PublicRes>PublicRes.xml</ofd:PublicRes>"));
        assert!(xml.contains("<ofd:DefaultCS>2</ofd:DefaultCS>"));
        assert!(xml.contains("TemplatePageName=\"bg\""));
    }

    #[test]
    fn test_ct_common_data_clone_debug() {
        let cd = CT_CommonData::new().max_unit_id(1);
        let cd2 = cd.clone();
        assert_eq!(cd2.get_max_unit_id(), Some(1));
        assert!(format!("{cd:?}").contains("CT_CommonData"));
    }
}
