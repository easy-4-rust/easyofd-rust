//! 将既有逐字绘制契约映射为 printpdf 操作流，保留字体子集及 Unicode 映射。
use std::cell::RefCell;
use std::io::{Read, Write};
use std::rc::Rc;

pub(crate) use printpdf::{
    BuiltinFont, Color, Line, LinePoint, Mm, Point, Rgb, TextMatrix, TextRenderingMode,
};
use printpdf::{
    Codepoint, Op, PaintMode, ParsedFont, PdfFontHandle, PdfPage, PdfSaveOptions, Polygon,
    PolygonRing, Pt, RawImage, RawImageData, RawImageFormat, TextItem, WindingOrder,
    XObjectTransform,
};

/// 文档与操作流共享句柄；仅在同步转换内部使用。
pub(crate) struct PdfDocumentReference(Rc<RefCell<printpdf::PdfDocument>>);
/// 页面绘制句柄，将绘制调用顺序保存在所属页面。
#[derive(Clone)]
pub(crate) struct PdfLayerReference {
    document: Rc<RefCell<printpdf::PdfDocument>>,
    page: usize,
}
/// 外部或内置字体的明确身份。
pub(crate) type IndirectFontRef = PdfFontHandle;
/// 文档构建入口，保留现有内部调用形式。
pub(crate) struct PdfDocument;
impl PdfDocument {
    /// 创建含首个页面的文档，返回文档、页号及内部层标记。
    pub(crate) fn create(
        title: &str,
        width: Mm,
        height: Mm,
        _layer: &str,
    ) -> (PdfDocumentReference, usize, usize) {
        let mut doc = printpdf::PdfDocument::new(title);
        doc.pages.push(PdfPage::new(width, height, Vec::new()));
        (PdfDocumentReference(Rc::new(RefCell::new(doc))), 0, 0)
    }
}
impl PdfDocumentReference {
    /// 返回页面操作句柄；参数是内部页号。
    pub(crate) fn get_page(&self, page: usize) -> PdfLayerReference {
        PdfLayerReference {
            document: Rc::clone(&self.0),
            page,
        }
    }
    /// 追加指定毫米尺寸的页面，返回内部页号及层标记。
    pub(crate) fn add_page(&self, width: Mm, height: Mm, _layer: &str) -> (usize, usize) {
        let mut doc = self.0.borrow_mut();
        let index = doc.pages.len();
        doc.pages.push(PdfPage::new(width, height, Vec::new()));
        (index, 0)
    }
    /// 注册内置字体；返回可直接供 Tf 使用的句柄。
    pub(crate) fn add_builtin_font(font: BuiltinFont) -> Result<IndirectFontRef, String> {
        Ok(PdfFontHandle::Builtin(font))
    }
    /// 解析字体并注册，参数读取器保留全部原字体字节供子集化使用。
    pub(crate) fn add_external_font(
        &self,
        mut reader: impl Read,
    ) -> Result<IndirectFontRef, String> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        let parsed = ParsedFont::from_bytes(&bytes, 0, &mut Vec::new())
            .ok_or_else(|| "字体解析失败".to_string())?;
        Ok(PdfFontHandle::External(
            self.0.borrow_mut().add_font(&parsed),
        ))
    }
    /// 加载外部字体；所有输出统一启用子集，参数用于兼容既有调用。
    pub(crate) fn add_external_font_with_subsetting(
        &self,
        reader: impl Read,
        _subset: bool,
    ) -> Result<IndirectFontRef, String> {
        self.add_external_font(reader)
    }
    /// 将操作流序列化并写入输出，保留子集化及真实位图。
    pub(crate) fn save(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let bytes = self.0.borrow().save(
            &PdfSaveOptions {
                optimize: false,
                subset_fonts: true,
                image_optimization: None,
                ..Default::default()
            },
            &mut Vec::new(),
        );
        writer.write_all(&bytes)?;
        writer.flush()
    }
}
impl PdfLayerReference {
    fn push(&self, operation: Op) {
        self.document.borrow_mut().pages[self.page]
            .ops
            .push(operation);
    }
    /// 返回当前页面绘制句柄；内部只使用单层。
    pub(crate) fn get_layer(self, _layer: usize) -> Self {
        self
    }
    /// 保存当前绘图状态。
    pub(crate) fn save_graphics_state(&self) {
        self.push(Op::SaveGraphicsState);
    }
    /// 恢复绘图状态。
    pub(crate) fn restore_graphics_state(&self) {
        self.push(Op::RestoreGraphicsState);
    }
    /// 设置填充颜色。
    pub(crate) fn set_fill_color(&self, col: Color) {
        self.push(Op::SetFillColor { col });
    }
    /// 设置描边颜色。
    pub(crate) fn set_outline_color(&self, col: Color) {
        self.push(Op::SetOutlineColor { col });
    }
    /// 设置以点为单位的描边宽度。
    pub(crate) fn set_outline_thickness(&self, pt: f32) {
        self.push(Op::SetOutlineThickness { pt: Pt(pt) });
    }
    /// 开始文本操作段。
    pub(crate) fn begin_text_section(&self) {
        self.push(Op::StartTextSection);
    }
    /// 结束文本操作段。
    pub(crate) fn end_text_section(&self) {
        self.push(Op::EndTextSection);
    }
    /// 设置字体与点字号。
    pub(crate) fn set_font(&self, font: &IndirectFontRef, size: f32) {
        self.push(Op::SetFont {
            font: font.clone(),
            size: Pt(size),
        });
    }
    /// 设置文字填充或描边模式。
    pub(crate) fn set_text_rendering_mode(&self, mode: TextRenderingMode) {
        self.push(Op::SetTextRenderingMode { mode });
    }
    /// 设置精确文本矩阵，保持 PDF 坐标与 CTM。
    pub(crate) fn set_text_matrix(&self, matrix: TextMatrix) {
        self.push(Op::SetTextMatrix { matrix });
    }
    /// 输出 Unicode 文本；字体已由 Tf 操作设置。
    pub(crate) fn write_text(&self, text: String, _font: &IndirectFontRef) {
        self.push(Op::ShowText {
            items: vec![TextItem::Text(text)],
        });
    }
    /// 输出显式字形并记录对应 Unicode，确保字形纳入字体子集。
    pub(crate) fn write_glyph(&self, gid: u16, character: char) {
        self.push(Op::ShowText {
            items: vec![TextItem::GlyphIds(vec![Codepoint::with_cid(
                gid,
                0.,
                character.to_string(),
            )])],
        });
    }
    /// 在毫米坐标处输出指定字号的文本。
    pub(crate) fn use_text(&self, text: &str, size: f32, x: Mm, y: Mm, font: &IndirectFontRef) {
        self.begin_text_section();
        self.set_font(font, size);
        self.push(Op::SetTextCursor {
            pos: Point::new(x, y),
        });
        self.write_text(text.to_string(), font);
        self.end_text_section();
    }
    /// 绘制折线或闭合轮廓。
    pub(crate) fn add_path(&self, line: Line, fill: bool) {
        if fill {
            self.push(Op::DrawPolygon {
                polygon: Polygon {
                    rings: vec![PolygonRing {
                        points: line.points,
                    }],
                    mode: PaintMode::FillStroke,
                    winding_order: WindingOrder::NonZero,
                },
            });
        } else {
            self.push(Op::DrawLine { line });
        }
    }
    /// 嵌入真实 RGBA 像素；尺寸及底部坐标从 OFD 毫米值换算。
    pub(crate) fn add_rgba_image(
        &self,
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        image: &easyofd_core::ImageObject,
        page_height: f64,
    ) {
        let raw = RawImage {
            pixels: RawImageData::U8(pixels),
            width: width as usize,
            height: height as usize,
            data_format: RawImageFormat::RGBA8,
            tag: Vec::new(),
        };
        let id = self.document.borrow_mut().add_image(&raw);
        self.push(Op::UseXobject {
            id,
            transform: XObjectTransform {
                translate_x: Some(Mm(image.x as f32).into()),
                translate_y: Some(Mm((page_height - image.y - image.height) as f32).into()),
                scale_x: Some(image.width as f32 * 300. / (width as f32 * 25.4)),
                scale_y: Some(image.height as f32 * 300. / (height as f32 * 25.4)),
                dpi: Some(300.),
                ..Default::default()
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{BuiltinFont, Color, Mm, PdfDocument, Rgb};
    use std::io::{Cursor, Read};

    fn fixture_font() -> Vec<u8> {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/upstream_20261014/stroke_true.ofd");
        let mut zip = zip::ZipArchive::new(Cursor::new(std::fs::read(file).unwrap())).unwrap();
        let mut font = Vec::new();
        zip.by_name("Doc_0/Res/font.ttf")
            .unwrap()
            .read_to_end(&mut font)
            .unwrap();
        font
    }

    #[test]
    fn explicit_glyph_is_subset_and_unicode_selectable() {
        let data = fixture_font();
        let face = ttf_parser::Face::parse(&data, 0).unwrap();
        let gid = face.glyph_index('/').unwrap().0;
        let (doc, page, layer) = PdfDocument::create("Glyph Unicode", Mm(100.), Mm(100.), "Layer");
        let font = doc.add_external_font(Cursor::new(data)).unwrap();
        let layer = doc.get_page(page).get_layer(layer);
        layer.begin_text_section();
        layer.set_font(&font, 12.);
        layer.write_glyph(gid, '中');
        layer.end_text_section();
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(pdf.extract_text(&[1]).unwrap().trim(), "中");
        assert!(
            pdf.objects.values().any(|object| object
                .as_stream()
                .is_ok_and(|stream| stream.dict.get(b"Length1").is_ok())),
            "subset font must be embedded"
        );
    }

    #[test]
    fn independent_pages_keep_their_dimensions_and_content() {
        let (doc, page, layer) = PdfDocument::create("Pages", Mm(100.), Mm(150.), "Layer");
        let font = super::PdfDocumentReference::add_builtin_font(BuiltinFont::Helvetica).unwrap();
        doc.get_page(page)
            .get_layer(layer)
            .use_text("first", 12., Mm(10.), Mm(20.), &font);
        let (page, layer) = doc.add_page(Mm(200.), Mm(250.), "Layer");
        doc.get_page(page)
            .get_layer(layer)
            .use_text("second", 12., Mm(10.), Mm(20.), &font);
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(pdf.get_pages().len(), 2);
        assert_eq!(pdf.extract_text(&[1]).unwrap().trim(), "first");
        assert_eq!(pdf.extract_text(&[2]).unwrap().trim(), "second");
        for (number, width) in [(1, 100.), (2, 200.)] {
            let id = pdf.get_pages()[&number];
            let bbox = pdf
                .get_object(id)
                .unwrap()
                .as_dict()
                .unwrap()
                .get(b"MediaBox")
                .unwrap()
                .as_array()
                .unwrap();
            assert!((f64::from(bbox[2].as_float().unwrap()) - width * 72. / 25.4).abs() < 0.01);
        }
    }

    #[test]
    fn filled_path_outputs_fill_and_stroke_operator() {
        let (doc, page, layer) = PdfDocument::create("Filled Path", Mm(100.), Mm(100.), "Layer");
        let layer = doc.get_page(page).get_layer(layer);
        layer.set_fill_color(Color::Rgb(Rgb::new(1., 0., 0., None)));
        let path = easyofd_core::PathObject::new(0., 0., "M 10 10 L 20 10 L 20 20 Z")
            .fill_color(0x00ff_0000);
        crate::render_path_to_pdf(&layer, &path, 100.);
        let mut bytes = Vec::new();
        doc.save(&mut bytes).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        let id = pdf.get_pages()[&1];
        let content = lopdf::content::Content::decode(&pdf.get_page_content(id)).unwrap();
        assert!(
            content
                .operations
                .iter()
                .any(|op| matches!(op.operator.as_str(), "B" | "B*" | "b" | "b*")),
            "filled path must actually paint fill and stroke"
        );
        let width = content
            .operations
            .iter()
            .find(|op| op.operator == "w")
            .unwrap()
            .operands[0]
            .as_float()
            .unwrap();
        assert!((f64::from(width) - path.stroke_width * 72. / 25.4).abs() < 0.001);
    }
}
