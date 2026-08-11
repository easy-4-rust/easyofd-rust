//! OFD XML 文档生成。
//!
//! 包含 OfdWriter 的 XML 构建方法，用于生成 GB/T 33190-2016 标准的 XML 文件。

use crate::OfdWriter;
use crate::helpers::xml_escape;
use easyofd_core::{ContentObject, ImageFormat, OfdPage};

impl OfdWriter {
    pub(crate) fn build_ofd_xml(&self) -> String {
        let mut xml = String::with_capacity(512);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&format!(
            r#"<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="{}">"#,
            self.options.metadata.version
        ));
        xml.push('\n');
        xml.push_str(r"  <ofd:DocBody>");
        xml.push('\n');
        xml.push_str(r"    <ofd:DocInfo>");
        xml.push('\n');

        if let Some(ref title) = self.options.metadata.title {
            xml.push_str(&format!(
                "      <ofd:Title>{}</ofd:Title>",
                xml_escape(title)
            ));
            xml.push('\n');
        }
        if let Some(ref author) = self.options.metadata.author {
            xml.push_str(&format!(
                "      <ofd:Author>{}</ofd:Author>",
                xml_escape(author)
            ));
            xml.push('\n');
        }
        if let Some(ref creator) = self.options.metadata.creator {
            xml.push_str(&format!(
                "      <ofd:Creator>{}</ofd:Creator>",
                xml_escape(creator)
            ));
            xml.push('\n');
        }
        if let Some(dt) = self.options.metadata.creation_date {
            xml.push_str(&format!(
                "      <ofd:CreationDate>{}</ofd:CreationDate>",
                dt.format("%Y-%m-%dT%H:%M:%S")
            ));
            xml.push('\n');
        }

        xml.push_str(r"    </ofd:DocInfo>");
        xml.push('\n');
        xml.push_str(r"    <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>");
        xml.push('\n');
        xml.push_str(r"  </ofd:DocBody>");
        xml.push('\n');
        xml.push_str(r"</ofd:OFD>");
        xml.push('\n');
        xml
    }

    pub(crate) fn build_document_xml(
        &self,
        image_resources: &[(String, &[u8], ImageFormat)],
    ) -> String {
        let mut xml = String::with_capacity(1024);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">"#);
        xml.push('\n');

        // Common Data
        xml.push_str(r"  <ofd:CommonData>");
        xml.push('\n');

        // Page area: use first page dimensions, or A4 default
        let (pw, ph) = self
            .pages
            .first()
            .map_or((210.0, 297.0), |p| (p.width, p.height));
        xml.push_str(&format!(
            r"    <ofd:PageArea><ofd:PhysicalBox>0 0 {pw:.2} {ph:.2}</ofd:PhysicalBox></ofd:PageArea>"
        ));
        xml.push('\n');

        // Font declarations
        xml.push_str(r"    <ofd:PublicRes>Doc_0/PublicRes.xml</ofd:PublicRes>");
        xml.push('\n');

        // Document resources
        if !image_resources.is_empty() {
            xml.push_str(r"    <ofd:DocumentRes>Doc_0/DocumentRes.xml</ofd:DocumentRes>");
            xml.push('\n');
        }

        xml.push_str(r"  </ofd:CommonData>");
        xml.push('\n');

        // Pages
        xml.push_str(r"  <ofd:Pages>");
        xml.push('\n');
        for i in 0..self.pages.len() {
            xml.push_str(&format!(
                r#"    <ofd:Page ID="{id}" BaseLoc="Pages/Page_{i}.xml"/>"#,
                id = i + 1
            ));
            xml.push('\n');
        }
        xml.push_str(r"  </ofd:Pages>");
        xml.push('\n');

        xml.push_str(r"</ofd:Document>");
        xml.push('\n');
        xml
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn build_document_res_xml(
        &self,
        image_resources: &[(String, &[u8], ImageFormat)],
    ) -> String {
        let mut xml = String::with_capacity(512);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:DocumentRes xmlns:ofd="http://www.ofdspec.org/2016">"#);
        xml.push('\n');
        xml.push_str(r"  <ofd:MultiMedia>");
        xml.push('\n');

        for (i, (res_name, _, fmt)) in image_resources.iter().enumerate() {
            let type_str = match fmt {
                ImageFormat::Jpeg => "JPEG",
                ImageFormat::Png => "PNG",
                ImageFormat::Bmp => "BMP",
                ImageFormat::Tiff => "TIFF",
            };
            // The BaseLoc is relative to the Doc_0 directory.
            let relative = res_name.strip_prefix("Doc_0/").unwrap_or(res_name);
            xml.push_str(&format!(
                r#"    <ofd:MultiMedia ID="{}" Type="{}"><ofd:MediaFile>{}</ofd:MediaFile></ofd:MultiMedia>"#,
                100 + i,
                type_str,
                relative,
            ));
            xml.push('\n');
        }

        xml.push_str(r"  </ofd:MultiMedia>");
        xml.push('\n');
        xml.push_str(r"</ofd:DocumentRes>");
        xml.push('\n');
        xml
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn build_public_res_xml(&self) -> String {
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8"?>"#,
            "\n",
            r#"<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">"#,
            "\n",
            "</ofd:Res>\n"
        )
        .to_string()
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn build_page_xml(
        &self,
        page: &OfdPage,
        page_index: usize,
        page_image_start: usize,
    ) -> String {
        let mut xml = String::with_capacity(2048);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&format!(
            r#"<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016" ID="{}">"#,
            page_index + 1
        ));
        xml.push('\n');

        // Page area
        xml.push_str(&format!(
            r"  <ofd:Area><ofd:PhysicalBox>0 0 {:.2} {:.2}</ofd:PhysicalBox></ofd:Area>",
            page.width, page.height
        ));
        xml.push('\n');

        // Content layer
        xml.push_str(r"  <ofd:Content>");
        xml.push('\n');

        // Collect image indices for this page.
        let mut image_counter = 0usize;

        for (object_index, obj) in page.content.iter().enumerate() {
            match obj {
                ContentObject::Text(text) => {
                    // mm to OFD units (1 mm = ~3.543307 pixels at 96dpi, but OFD uses mm directly)
                    let x = text.x;
                    let y = text.y;
                    // Estimate text width: ~0.3mm per character for 12pt SimSun (rough heuristic)
                    let character_count =
                        f64::from(u32::try_from(text.text.chars().count()).unwrap_or(u32::MAX));
                    let est_width = text.width.unwrap_or(character_count * text.size * 0.06);
                    let est_height = text.height.unwrap_or(text.size * 0.4);

                    xml.push_str(&format!(
                        r#"    <ofd:TextObject ID="t_{page_index}_{idx}" Boundary="{x:.2} {y:.2} {w:.2} {h:.2}" Font="{font}" Size="{size:.1}">"#,
                        idx = page_index * 1000 + object_index,
                        w = est_width,
                        h = est_height,
                        font = text.font,
                        size = text.size,
                    ));
                    xml.push('\n');

                    // TextCode
                    xml.push_str(&format!(
                        r#"      <ofd:TextCode X="0" Y="{y:.2}">{text}</ofd:TextCode>"#,
                        y = text.size * 0.8,
                        text = xml_escape(&text.text),
                    ));
                    xml.push('\n');

                    xml.push_str(r"    </ofd:TextObject>");
                    xml.push('\n');
                }
                ContentObject::Image(img) => {
                    // Find the resource ID for this image.
                    let global_image_index = page_image_start + image_counter;
                    let res_id = 100 + global_image_index;

                    xml.push_str(&format!(
                        r#"    <ofd:ImageObject ID="i_{page_index}_{idx}" Boundary="{x:.2} {y:.2} {w:.2} {h:.2}" ResourceID="{res_id}"/>"#,
                        idx = page_index * 1000 + object_index,
                        x = img.x,
                        y = img.y,
                        w = img.width,
                        h = img.height,
                    ));
                    xml.push('\n');
                    image_counter += 1;
                }
                ContentObject::Path(path) => {
                    let stroke = format!("{:06X}", path.stroke_color);
                    xml.push_str(&format!(
                        r#"    <ofd:PathObject ID="p_{page_index}_{idx}" Boundary="{x:.2} {y:.2} 0 0" StrokeColor="{stroke}" LineWidth="{lw:.2}">"#,
                        idx = page_index * 1000 + object_index,
                        x = path.x,
                        y = path.y,
                        lw = path.stroke_width,
                    ));
                    xml.push('\n');
                    xml.push_str(&format!(
                        r"      <ofd:AbbreviatedData>{}</ofd:AbbreviatedData>",
                        xml_escape(&path.path_data),
                    ));
                    xml.push('\n');
                    xml.push_str(r"    </ofd:PathObject>");
                    xml.push('\n');
                }
            }
        }

        xml.push_str(r"  </ofd:Content>");
        xml.push('\n');
        xml.push_str(r"</ofd:Page>");
        xml.push('\n');
        xml
    }
}
