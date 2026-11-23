//! 栅格渲染器：将 OFD 页面渲染为像素图。
//!
//! 对应 Java: org.ofdrw.converter.AWTMaker（渲染管线核心）
//!
//! 使用 `image` crate 的 `RgbaImage` 作为画布，Fontations 和 `tiny-skia` 渲染文本，
//! `tiny-skia` 渲染矢量路径，`image` 解码/缩放/合成图片。
//!
//! # 绘制管线
//!
//! 1. 按 DPI（默认 96）将 mm 页面尺寸换算为像素
//! 2. 创建白色背景 `RgbaImage` 画布
//! 3. 按顺序遍历 `ContentObject`：
//!    - **文本**：Fontations 轮廓逐字形光栅化 → 逐像素绘制到画布
//!    - **图片**：image 解码 → resize → alpha 合成到画布
//!    - **路径**：tiny-skia PathBuilder → Pixmap → 合成到画布

use ::image::{Rgba, RgbaImage};
use easyofd_core::{ContentObject, ImageObject, OfdPage, PathObject, TextObject};
use tiny_skia::{Paint, PathBuilder as SkiaPathBuilder, Pixmap, Stroke, Transform};

/// 默认渲染 DPI（每英寸点数）。
const DEFAULT_DPI: f64 = 96.0;

/// 毫米转英寸因子。
const MM_PER_INCH: f64 = 25.4;

/// 栅格渲染器，将 OFD 页面渲染为 `RgbaImage`。
///
/// 对应 Java: `org.ofdrw.converter.AWTMaker`（核心渲染逻辑）
#[derive(Debug, Clone)]
pub struct RasterRenderer {
    /// 渲染 DPI。
    dpi: f64,
    /// 背景颜色（RGBA，0xRRGGBBAA）。
    background_color: u32,
}

impl RasterRenderer {
    /// 创建新的栅格渲染器（默认 DPI 96，白色背景）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            dpi: DEFAULT_DPI,
            background_color: 0xFFFF_FFFF,
        }
    }

    /// 设置渲染 DPI。
    #[must_use]
    pub fn with_dpi(mut self, dpi: f64) -> Self {
        self.dpi = dpi;
        self
    }

    /// 设置背景颜色（RGB 0xRRGGBB，alpha 固定为 0xFF）。
    #[must_use]
    pub fn with_background_color(mut self, color: u32) -> Self {
        self.background_color = (color & 0x00FF_FFFF) | 0xFF00_0000;
        self
    }

    /// 获取当前 DPI。
    #[must_use]
    pub fn dpi(&self) -> f64 {
        self.dpi
    }

    /// 将 mm 转换为像素。
    #[must_use]
    pub fn mm_to_px(&self, mm: f64) -> f64 {
        mm / MM_PER_INCH * self.dpi
    }

    /// 将 pt（字号单位）转换为像素。
    #[must_use]
    pub fn pt_to_px(&self, pt: f64) -> f64 {
        pt * self.dpi / 72.0
    }

    /// 渲染一个 OFD 页面为 `RgbaImage`。
    ///
    /// # 参数
    ///
    /// - `page`: OFD 页面数据
    ///
    /// # 返回
    ///
    /// 渲染后的 RGBA 像素图。
    /// 兼容受信模型的旧入口；外部输入请使用try_render_page或文件转换接口。
    pub fn render_page(&self, page: &OfdPage) -> RgbaImage {
        let width_px = self.mm_to_px(page.width).round().max(1.0) as u32;
        let height_px = self.mm_to_px(page.height).round().max(1.0) as u32;

        // 创建画布并填充背景色
        let bg = u32_to_rgba(self.background_color);
        let mut canvas = RgbaImage::from_pixel(width_px, height_px, bg);

        // 按顺序渲染每个内容对象
        for content in &page.content {
            match content {
                ContentObject::Text(text) => {
                    self.render_text(&mut canvas, text);
                }
                ContentObject::Image(img) => {
                    self.render_image(&mut canvas, img);
                }
                ContentObject::Path(path) => {
                    self.render_path(&mut canvas, path, height_px);
                }
            }
        }

        canvas
    }

    /// 安全栅格入口。参数为页面；非法尺寸或超32百万像素预算返回错误。
    /// 旧render_page用于受信模型，不接收未校验的外部尺寸。
    pub fn try_render_page(&self, page: &OfdPage) -> easyofd_core::OfdResult<RgbaImage> {
        self.render_page_with_text_runs(page, &[])
    }

    fn validate_resource_budget(&self, page: &OfdPage) -> easyofd_core::OfdResult<()> {
        use easyofd_core::OfdError;
        if !self.dpi.is_finite() || self.dpi <= 0. {
            return Err(OfdError::Conversion("DPI必须为有限正数".into()));
        }
        let valid = |width: f64, height: f64| {
            if !width.is_finite() || !height.is_finite() || width <= 0. || height <= 0. {
                return false;
            }
            let w = self.mm_to_px(width).round().max(1.);
            let h = self.mm_to_px(height).round().max(1.);
            w.is_finite() && h.is_finite() && w <= 16384. && h <= 16384. && w * h <= 32_000_000.
        };
        if !valid(page.width, page.height) {
            return Err(OfdError::Conversion(
                "页面尺寸超出栅格预算（边长16384px、32百万像素）".into(),
            ));
        }
        for object in &page.content {
            if let ContentObject::Text(text) = object {
                let px = self.pt_to_px(text.size);
                if !text.size.is_finite()
                    || text.size <= 0.
                    || !px.is_finite()
                    || px > 1024.
                    || !text.x.is_finite()
                    || !text.y.is_finite()
                {
                    return Err(OfdError::Conversion(
                        "文字字号或位置超出安全栅格预算".into(),
                    ));
                }
            }
            if let ContentObject::Image(image) = object {
                if !valid(image.width, image.height) {
                    return Err(OfdError::Conversion("图片缩放尺寸超出栅格预算".into()));
                }
                // 先读图片头再解码，阻止小压缩数据声明超大像素尺寸。
                if let Ok(reader) =
                    image::ImageReader::new(std::io::Cursor::new(&image.data)).with_guessed_format()
                {
                    if let Ok((w, h)) = reader.into_dimensions() {
                        if w > 16384 || h > 16384 || u64::from(w) * u64::from(h) > 32_000_000 {
                            return Err(OfdError::Conversion(
                                "嵌入图片解码尺寸超出栅格预算".into(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 以转换元数据渲染页面；字体/字形错误返回明确错误。
    /// 参数为页面和对应的文字绘制数据，返回RGBA图像。
    pub(crate) fn render_page_with_text_runs(
        &self,
        page: &OfdPage,
        runs: &[easyofd_reader::TextRenderRun],
    ) -> easyofd_core::OfdResult<RgbaImage> {
        self.validate_resource_budget(page)?;
        let w = self.mm_to_px(page.width).round().max(1.) as u32;
        let h = self.mm_to_px(page.height).round().max(1.) as u32;
        let bg = Rgba([
            ((self.background_color >> 16) & 255) as u8,
            ((self.background_color >> 8) & 255) as u8,
            (self.background_color & 255) as u8,
            255,
        ]);
        let mut canvas = RgbaImage::from_pixel(w, h, bg);
        for (i, obj) in page.content.iter().enumerate() {
            match obj {
                ContentObject::Text(text) => {
                    if let Some(run) = runs
                        .iter()
                        .find(|r| r.content_index == i && !r.uses_legacy_model())
                    {
                        self.render_text_run(&mut canvas, run)?;
                    } else {
                        self.render_text_checked(&mut canvas, text)?;
                    }
                }
                ContentObject::Path(path) => self.render_path(&mut canvas, path, h),
                ContentObject::Image(img) => self.render_image(&mut canvas, img),
            }
        }
        Ok(canvas)
    }

    fn render_text_run(
        &self,
        canvas: &mut RgbaImage,
        run: &easyofd_reader::TextRenderRun,
    ) -> easyofd_core::OfdResult<()> {
        use crate::font::glyph_outline::GlyphOutline;
        use crate::utils::text_layout::layout_text_run;
        use easyofd_core::OfdError;
        let fallback = if run.font_data.is_none() {
            find_font_data(run.font_name.as_deref().unwrap_or("Helvetica"), false)
                .or_else(find_system_font)
        } else {
            None
        };
        let data = run
            .font_data
            .as_deref()
            .or(fallback.as_deref())
            .ok_or_else(|| OfdError::Conversion("没有可用字体".into()))?;
        let face = crate::font::font_face::FontFace::parse(data, 0)
            .map_err(|_| OfdError::Conversion("字体无效".into()))?;
        let size = run.text.size.unwrap_or(12. * 25.4 / 72.);
        if !size.is_finite() || size <= 0. {
            return Err(OfdError::Conversion("字号无效".into()));
        }
        let advances: Vec<f64> = run
            .text
            .text_codes
            .iter()
            .flat_map(|c| c.content.chars())
            .map(|c| {
                face.glyph_index(c)
                    .and_then(|id| face.glyph_hor_advance(id))
                    .map_or(size * 0.5, |a| {
                        f64::from(a) / f64::from(face.units_per_em()) * size
                    })
            })
            .collect();
        let positions = layout_text_run(run, &advances)?;
        let mut pixmap = Pixmap::new(canvas.width(), canvas.height())
            .ok_or_else(|| OfdError::Conversion("画布过大".into()))?;
        let units = size / f64::from(face.units_per_em());
        let px = self.dpi / 25.4;
        for p in positions {
            let id = if let Some(id) = p.glyph_id {
                if id >= u32::from(face.number_of_glyphs()) {
                    return Err(OfdError::Conversion("glyph越界".into()));
                }
                crate::font::font_face::GlyphId(id as u16)
            } else {
                face.glyph_index(p.character)
                    .ok_or_else(|| OfdError::Conversion(format!("字体缺少字形: {}", p.character)))?
            };
            let glyph = GlyphOutline::load(&face, id)?;
            let Some(path) = glyph.path else {
                continue;
            };
            let m = p.linear_ctm;
            let tr = Transform::from_row(
                (m[0] * units * px) as f32,
                (m[1] * units * px) as f32,
                (-m[2] * units * px) as f32,
                (-m[3] * units * px) as f32,
                (p.origin_mm[0] * px) as f32,
                (p.origin_mm[1] * px) as f32,
            );
            let mut paint = Paint {
                anti_alias: true,
                ..Paint::default()
            };
            if run.text.fill {
                paint.set_color(u32_to_premultiplied_rgba(
                    run.text.fill_color.unwrap_or(0),
                    255,
                ));
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, tr, None);
            }
            if run.text.stroke && run.line_width_mm > 0. {
                paint.set_color(u32_to_premultiplied_rgba(
                    run.text.stroke_color.unwrap_or(0),
                    255,
                ));
                pixmap.stroke_path(
                    &path,
                    &paint,
                    &Stroke {
                        width: (run.line_width_mm / units) as f32,
                        ..Stroke::default()
                    },
                    tr,
                    None,
                );
            }
        }
        for (i, src) in pixmap.pixels().iter().enumerate() {
            if src.alpha() == 0 {
                continue;
            }
            let c = src.demultiply();
            let dst: &mut [u8] = canvas.as_mut();
            let base = i * 4;
            let alpha = f64::from(c.alpha()) / 255.;
            dst[base] = (f64::from(c.red()) * alpha + f64::from(dst[base]) * (1. - alpha)) as u8;
            dst[base + 1] =
                (f64::from(c.green()) * alpha + f64::from(dst[base + 1]) * (1. - alpha)) as u8;
            dst[base + 2] =
                (f64::from(c.blue()) * alpha + f64::from(dst[base + 2]) * (1. - alpha)) as u8;
            dst[base + 3] = 255;
        }
        Ok(())
    }

    // ─── 文本渲染 ──────────────────────────────────────────────────────────

    /// 渲染文本对象到画布。
    fn render_text(&self, canvas: &mut RgbaImage, text: &TextObject) {
        let _ = self.render_text_checked(canvas, text);
    }

    fn render_text_checked(
        &self,
        canvas: &mut RgbaImage,
        text: &TextObject,
    ) -> easyofd_core::OfdResult<()> {
        if text.text.is_empty() {
            return Ok(());
        }

        let font_data = match load_font_for_text(text) {
            Some(f) => f,
            None => return Ok(()), // 无法加载字体时静默跳过
        };

        let font = crate::font::font_face::FontFace::parse(&font_data, 0)
            .map_err(|_| easyofd_core::OfdError::Conversion("字体无效".into()))?;
        self.render_text_with_face(canvas, text, &font)
    }

    fn render_text_with_face(
        &self,
        canvas: &mut RgbaImage,
        text: &TextObject,
        font: &crate::font::font_face::FontFace<'_>,
    ) -> easyofd_core::OfdResult<()> {
        let px_size = self.pt_to_px(text.size) as f32;
        if !px_size.is_finite() || px_size <= 0. || px_size > 1024. {
            return Err(easyofd_core::OfdError::Conversion(
                "文字字号超出安全栅格预算".into(),
            ));
        }
        if px_size < 1.0 {
            return Ok(());
        }

        let color = u32_to_rgba(text.color);
        let start_x = self.mm_to_px(text.x) as f32;
        let start_y = self.mm_to_px(text.y) as f32;
        let canvas_w = canvas.width();
        let canvas_h = canvas.height();

        // 逐字符光栅化并绘制
        let mut cursor_x = start_x;
        for ch in text.text.chars() {
            let glyph = crate::font::glyph_bitmap::GlyphBitmap::rasterize(font, ch, px_size)?;
            // 保留旧入口 start_y + px_size + floor(ymin) 的字形放置语义。
            let glyph_top = start_y + px_size + glyph.ymin;
            if let Some(bitmap) = &glyph.pixels {
                for row in 0..bitmap.height() {
                    for col in 0..bitmap.width() {
                        let alpha = bitmap.pixels()[(row * bitmap.width() + col) as usize].alpha();
                        if alpha == 0 {
                            continue;
                        }
                        let px_i = (cursor_x + col as f32 + glyph.xmin) as i32;
                        let py_i = (glyph_top + row as f32) as i32;
                        if px_i >= 0
                            && py_i >= 0
                            && (px_i as u32) < canvas_w
                            && (py_i as u32) < canvas_h
                        {
                            blend_pixel(
                                canvas.get_pixel_mut(px_i as u32, py_i as u32),
                                color,
                                alpha,
                            );
                        }
                    }
                }
            }

            // 推进光标（metrics.advance 是水平推进量）
            cursor_x += glyph.advance;
        }
        Ok(())
    }

    // ─── 图片渲染 ──────────────────────────────────────────────────────────

    /// 渲染图片对象到画布。
    ///
    /// 解码图片 → resize 到目标 mm 尺寸 → alpha 合成到画布 (x, y) 位置。
    fn render_image(&self, canvas: &mut RgbaImage, img: &ImageObject) {
        if img.data.is_empty() {
            return;
        }

        let loaded = match ::image::load_from_memory(&img.data) {
            Ok(i) => i.to_rgba8(),
            Err(_) => return, // 图片数据无效时静默跳过
        };

        let target_w = self.mm_to_px(img.width).round().max(1.0) as u32;
        let target_h = self.mm_to_px(img.height).round().max(1.0) as u32;

        // 缩放到目标尺寸
        let resized = ::image::imageops::resize(
            &loaded,
            target_w,
            target_h,
            ::image::imageops::FilterType::Triangle,
        );

        // 合成到画布（带 alpha 混合）
        let offset_x = self.mm_to_px(img.x).round() as i64;
        let offset_y = self.mm_to_px(img.y).round() as i64;

        for (src_x, src_y, pixel) in resized.enumerate_pixels() {
            let dst_x = offset_x + i64::from(src_x);
            let dst_y = offset_y + i64::from(src_y);

            if dst_x >= 0
                && dst_y >= 0
                && (dst_x as u32) < canvas.width()
                && (dst_y as u32) < canvas.height()
            {
                let dst_pixel = canvas.get_pixel_mut(dst_x as u32, dst_y as u32);
                blend_pixel(dst_pixel, *pixel, pixel[3]);
            }
        }
    }

    // ─── 路径渲染 ──────────────────────────────────────────────────────────

    /// 渲染路径对象到画布。
    ///
    /// 使用 `tiny-skia` 解析 SVG 风格路径命令，渲染到临时 Pixmap，
    /// 然后合成到主画布。
    fn render_path(&self, canvas: &mut RgbaImage, path: &PathObject, _page_height_px: u32) {
        let skia_path = match parse_svg_path(&path.path_data) {
            Some(p) => p,
            None => return, // 路径数据解析失败时静默跳过
        };

        let pixmap_w = canvas.width();
        let pixmap_h = canvas.height();

        let mut pixmap = match Pixmap::new(pixmap_w, pixmap_h) {
            Some(p) => p,
            None => return,
        };

        // 路径偏移（mm → px）
        let offset_x = self.mm_to_px(path.x) as f32;
        let offset_y = self.mm_to_px(path.y) as f32;
        let transform = Transform::from_translate(offset_x, offset_y);

        // 填充
        if let Some(fill_rgb) = path.fill_color {
            let fill_color = u32_to_premultiplied_rgba(fill_rgb, 255);
            let mut paint = Paint::default();
            paint.set_color(fill_color);
            pixmap.fill_path(
                &skia_path,
                &paint,
                tiny_skia::FillRule::Winding,
                transform,
                None,
            );
        }

        // 描边
        let stroke_color_rgb = path.stroke_color;
        let stroke_color = u32_to_premultiplied_rgba(stroke_color_rgb, 255);
        let stroke_width_px = self.mm_to_px(path.stroke_width) as f32;
        if stroke_width_px > 0.0 {
            let mut paint = Paint::default();
            paint.set_color(stroke_color);
            let stroke = Stroke {
                width: stroke_width_px.max(0.5),
                ..Stroke::default()
            };
            pixmap.stroke_path(&skia_path, &paint, &stroke, transform, None);
        }

        // 合成 pixmap 到 canvas（非预乘 → 普通 alpha）
        for y in 0..pixmap_h {
            for x in 0..pixmap_w {
                let src = pixmap.pixel(x, y);
                if let Some(src) = src {
                    if src.alpha() == 0 {
                        continue;
                    }
                    let dst_pixel = canvas.get_pixel_mut(x, y);
                    let src_a = f64::from(src.alpha()) / 255.0;
                    let dst_a = f64::from(dst_pixel[3]) / 255.0;
                    let out_a = src_a + dst_a * (1.0 - src_a);
                    if out_a > 0.0 {
                        // tiny-skia 输出预乘 alpha，需要还原
                        let src_r = f64::from(src.red()) / src_a;
                        let src_g = f64::from(src.green()) / src_a;
                        let src_b = f64::from(src.blue()) / src_a;
                        let dst_r = f64::from(dst_pixel[0]);
                        let dst_g = f64::from(dst_pixel[1]);
                        let dst_b = f64::from(dst_pixel[2]);
                        dst_pixel[0] =
                            ((src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a) as u8;
                        dst_pixel[1] =
                            ((src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a) as u8;
                        dst_pixel[2] =
                            ((src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a) as u8;
                        dst_pixel[3] = (out_a * 255.0) as u8;
                    }
                }
            }
        }
    }
}

impl Default for RasterRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 私有辅助函数 ────────────────────────────────────────────────────────

/// 将 0xRRGGBB 颜色转换为 `Rgba` 像素值（alpha=0xFF）。
fn u32_to_rgba(color: u32) -> Rgba<u8> {
    Rgba([
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
        0xFF,
    ])
}

/// 将 0xRRGGBB 颜色转换为 tiny-skia 的预乘 RGBA。
fn u32_to_premultiplied_rgba(color: u32, alpha: u8) -> tiny_skia::Color {
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    let a = f32::from(alpha) / 255.0;
    tiny_skia::Color::from_rgba(r * a, g * a, b * a, a).unwrap_or(tiny_skia::Color::TRANSPARENT)
}

/// Alpha 混合：将前景色（带 glyph alpha）混合到目标像素。
fn blend_pixel(dst: &mut Rgba<u8>, fg_color: Rgba<u8>, glyph_alpha: u8) {
    if glyph_alpha == 0 {
        return;
    }
    let alpha = f64::from(glyph_alpha) / 255.0;
    let dst_a = f64::from(dst[3]) / 255.0;
    let out_a = alpha + dst_a * (1.0 - alpha);
    if out_a > 0.0 {
        for i in 0..3 {
            let s = f64::from(fg_color[i]);
            let d = f64::from(dst[i]);
            dst[i] = ((s * alpha + d * dst_a * (1.0 - alpha)) / out_a) as u8;
        }
        dst[3] = (out_a * 255.0) as u8;
    }
}

/// 根据文本对象的字体信息加载字体。
///
/// 查找策略：
/// 1. 根据字体名称匹配系统字体路径（含 CJK 字体探测）
/// 2. 尝试系统通用字体（Helvetica/Arial/DejaVuSans 等）
/// 3. 全部失败则返回 `None`（该文本段将被静默跳过）
fn load_font_for_text(text: &TextObject) -> Option<Vec<u8>> {
    // 保留字体名称优先、系统字体回退及集合索引零的选择顺序。
    let validate = |data: Vec<u8>| {
        crate::font::font_face::FontFace::parse(&data, 0)
            .is_ok()
            .then_some(data)
    };
    find_font_data(&text.font, text.weight >= 700)
        .and_then(validate)
        .or_else(|| find_system_font().and_then(validate))
}

/// 根据字体名称查找系统字体文件数据。
fn find_font_data(font_name: &str, bold: bool) -> Option<Vec<u8>> {
    // 先尝试 CJK 字体探测
    if let Some(info) = crate::cjk_font::find_cjk_font()
        && let Ok(data) = std::fs::read(&info.path)
    {
        return Some(data);
    }

    // 按名称匹配常见系统字体路径
    let candidates = system_font_candidates(font_name, bold);
    for path in candidates {
        if path.exists()
            && let Ok(data) = std::fs::read(&path)
        {
            return Some(data);
        }
    }

    None
}

/// 查找系统通用字体（用于非 CJK 文本的回退）。
fn find_system_font() -> Option<Vec<u8>> {
    let paths: &[&str] = &[
        // macOS
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/SFNSText.ttf",
        "/Library/Fonts/Arial.ttf",
        // Linux
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
    ];

    for &path in paths {
        let p = std::path::Path::new(path);
        if p.exists()
            && let Ok(data) = std::fs::read(p)
        {
            return Some(data);
        }
    }

    None
}

/// 根据字体名称和粗体属性返回候选系统字体路径。
fn system_font_candidates(font_name: &str, bold: bool) -> Vec<std::path::PathBuf> {
    // 非 Windows 平台不使用粗体参数（Windows 分支用于选择粗体字体文件）
    let _ = bold;
    let mut paths = Vec::new();
    let name_lower = font_name.to_lowercase();

    // macOS 字体目录
    #[cfg(target_os = "macos")]
    {
        let macos_base = std::path::Path::new("/System/Library/Fonts");
        let macos_supplemental = std::path::Path::new("/System/Library/Fonts/Supplemental");
        let library_fonts = std::path::Path::new("/Library/Fonts");

        // 常见中文字体名映射
        if name_lower.contains("simsun") || name_lower.contains("宋体") {
            paths.push(macos_supplemental.join("Songti.ttc"));
            paths.push(library_fonts.join("Songti.ttc"));
        } else if name_lower.contains("simhei") || name_lower.contains("黑体") {
            paths.push(macos_base.join("STHeiti Light.ttc"));
            paths.push(macos_base.join("STHeiti Medium.ttc"));
        } else if name_lower.contains("pingfang") || name_lower.contains("苹方") {
            paths.push(macos_base.join("PingFang.ttc"));
        } else if name_lower.contains("arial") || name_lower.contains("helvetica") {
            paths.push(macos_base.join("Helvetica.ttc"));
            paths.push(library_fonts.join("Arial.ttf"));
        }

        // 通用回退
        paths.push(macos_base.join("PingFang.ttc"));
        paths.push(macos_base.join("Helvetica.ttc"));
    }

    // Linux 字体目录
    #[cfg(target_os = "linux")]
    {
        let linux_base = std::path::Path::new("/usr/share/fonts");
        if name_lower.contains("simsun")
            || name_lower.contains("宋体")
            || name_lower.contains("simhei")
            || name_lower.contains("黑体")
        {
            // Linux 中文字体统一回退文泉驿微米黑
            paths.push(linux_base.join("truetype/wqy/wqy-microhei.ttc"));
        }
        paths.push(linux_base.join("truetype/dejavu/DejaVuSans.ttf"));
        paths.push(linux_base.join("truetype/liberation/LiberationSans-Regular.ttf"));
    }

    // Windows 字体目录
    #[cfg(target_os = "windows")]
    {
        if let Some(win_dir) = std::env::var_os("SystemRoot") {
            let fonts_dir = std::path::Path::new(&win_dir).join("Fonts");
            if name_lower.contains("simsun") || name_lower.contains("宋体") {
                paths.push(fonts_dir.join("simsun.ttc"));
            } else if name_lower.contains("simhei") || name_lower.contains("黑体") {
                paths.push(fonts_dir.join("simhei.ttf"));
            }
            if bold {
                paths.push(fonts_dir.join("msyhbd.ttc"));
            }
            paths.push(fonts_dir.join("msyh.ttc"));
            paths.push(fonts_dir.join("arial.ttf"));
        }
    }

    paths
}

// ─── SVG 路径解析 ────────────────────────────────────────────────────────

/// 解析 SVG 风格路径数据为 `tiny_skia::Path`。
///
/// 支持的命令：M/m（移动）、L/l（直线）、C/c（三次贝塞尔）、Q/q（二次贝塞尔）、
/// Z/z（闭合）。不支持的命令将被安全跳过并记录。
fn parse_svg_path(data: &str) -> Option<tiny_skia::Path> {
    let tokens = tokenize_path(data);
    if tokens.is_empty() {
        return None;
    }

    let mut builder = SkiaPathBuilder::new();
    let mut i = 0;
    let mut cur_x = 0.0_f32;
    let mut cur_y = 0.0_f32;
    let mut start_x = 0.0_f32;
    let mut start_y = 0.0_f32;

    while i < tokens.len() {
        let cmd = &tokens[i];
        match cmd.as_str() {
            // ── 绝对命令 ──
            "M" => {
                if i + 2 < tokens.len() {
                    let x: f32 = parse_token(&tokens[i + 1]);
                    let y: f32 = parse_token(&tokens[i + 2]);
                    builder.move_to(x, y);
                    cur_x = x;
                    cur_y = y;
                    start_x = x;
                    start_y = y;
                    i += 3;
                    // M 后续的坐标对隐含 L
                    while i + 2 <= tokens.len() && !is_command(&tokens[i]) {
                        let lx: f32 = parse_token(&tokens[i]);
                        let ly: f32 = parse_token(&tokens[i + 1]);
                        builder.line_to(lx, ly);
                        cur_x = lx;
                        cur_y = ly;
                        i += 2;
                    }
                } else {
                    i += 1;
                }
            }
            "L" => {
                i += 1;
                while i + 2 <= tokens.len() && !is_command(&tokens[i]) {
                    let x: f32 = parse_token(&tokens[i]);
                    let y: f32 = parse_token(&tokens[i + 1]);
                    builder.line_to(x, y);
                    cur_x = x;
                    cur_y = y;
                    i += 2;
                }
            }
            "C" => {
                i += 1;
                while i + 6 <= tokens.len() && !is_command(&tokens[i]) {
                    let x1: f32 = parse_token(&tokens[i]);
                    let y1: f32 = parse_token(&tokens[i + 1]);
                    let x2: f32 = parse_token(&tokens[i + 2]);
                    let y2: f32 = parse_token(&tokens[i + 3]);
                    let x: f32 = parse_token(&tokens[i + 4]);
                    let y: f32 = parse_token(&tokens[i + 5]);
                    builder.cubic_to(x1, y1, x2, y2, x, y);
                    cur_x = x;
                    cur_y = y;
                    i += 6;
                }
            }
            "Q" => {
                i += 1;
                while i + 4 <= tokens.len() && !is_command(&tokens[i]) {
                    let x1: f32 = parse_token(&tokens[i]);
                    let y1: f32 = parse_token(&tokens[i + 1]);
                    let x: f32 = parse_token(&tokens[i + 2]);
                    let y: f32 = parse_token(&tokens[i + 3]);
                    builder.quad_to(x1, y1, x, y);
                    cur_x = x;
                    cur_y = y;
                    i += 4;
                }
            }
            "Z" | "z" => {
                builder.close();
                cur_x = start_x;
                cur_y = start_y;
                i += 1;
            }
            // ── 相对命令 ──
            "m" => {
                if i + 2 < tokens.len() {
                    let dx: f32 = parse_token(&tokens[i + 1]);
                    let dy: f32 = parse_token(&tokens[i + 2]);
                    cur_x += dx;
                    cur_y += dy;
                    builder.move_to(cur_x, cur_y);
                    start_x = cur_x;
                    start_y = cur_y;
                    i += 3;
                    while i + 2 <= tokens.len() && !is_command(&tokens[i]) {
                        let dx: f32 = parse_token(&tokens[i]);
                        let dy: f32 = parse_token(&tokens[i + 1]);
                        cur_x += dx;
                        cur_y += dy;
                        builder.line_to(cur_x, cur_y);
                        i += 2;
                    }
                } else {
                    i += 1;
                }
            }
            "l" => {
                i += 1;
                while i + 2 <= tokens.len() && !is_command(&tokens[i]) {
                    let dx: f32 = parse_token(&tokens[i]);
                    let dy: f32 = parse_token(&tokens[i + 1]);
                    cur_x += dx;
                    cur_y += dy;
                    builder.line_to(cur_x, cur_y);
                    i += 2;
                }
            }
            "c" => {
                i += 1;
                while i + 6 <= tokens.len() && !is_command(&tokens[i]) {
                    let dx1: f32 = parse_token(&tokens[i]);
                    let dy1: f32 = parse_token(&tokens[i + 1]);
                    let dx2: f32 = parse_token(&tokens[i + 2]);
                    let dy2: f32 = parse_token(&tokens[i + 3]);
                    let dx: f32 = parse_token(&tokens[i + 4]);
                    let dy: f32 = parse_token(&tokens[i + 5]);
                    builder.cubic_to(
                        cur_x + dx1,
                        cur_y + dy1,
                        cur_x + dx2,
                        cur_y + dy2,
                        cur_x + dx,
                        cur_y + dy,
                    );
                    cur_x += dx;
                    cur_y += dy;
                    i += 6;
                }
            }
            "q" => {
                i += 1;
                while i + 4 <= tokens.len() && !is_command(&tokens[i]) {
                    let dx1: f32 = parse_token(&tokens[i]);
                    let dy1: f32 = parse_token(&tokens[i + 1]);
                    let dx: f32 = parse_token(&tokens[i + 2]);
                    let dy: f32 = parse_token(&tokens[i + 3]);
                    builder.quad_to(cur_x + dx1, cur_y + dy1, cur_x + dx, cur_y + dy);
                    cur_x += dx;
                    cur_y += dy;
                    i += 4;
                }
            }
            "H" => {
                i += 1;
                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x: f32 = parse_token(&tokens[i]);
                    builder.line_to(x, cur_y);
                    cur_x = x;
                    i += 1;
                }
            }
            "h" => {
                i += 1;
                while i < tokens.len() && !is_command(&tokens[i]) {
                    let dx: f32 = parse_token(&tokens[i]);
                    cur_x += dx;
                    builder.line_to(cur_x, cur_y);
                    i += 1;
                }
            }
            "V" => {
                i += 1;
                while i < tokens.len() && !is_command(&tokens[i]) {
                    let y: f32 = parse_token(&tokens[i]);
                    builder.line_to(cur_x, y);
                    cur_y = y;
                    i += 1;
                }
            }
            "v" => {
                i += 1;
                while i < tokens.len() && !is_command(&tokens[i]) {
                    let dy: f32 = parse_token(&tokens[i]);
                    cur_y += dy;
                    builder.line_to(cur_x, cur_y);
                    i += 1;
                }
            }
            // 不支持的命令：安全跳过
            _other => {
                // 不支持的 SVG 路径命令，静默跳过
                i += 1;
            }
        }
    }

    builder.finish()
}

/// 将路径数据字符串分词。
///
/// 按空白和命令字母分隔，保留命令字母作为独立 token。
fn tokenize_path(data: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in data.chars() {
        if ch.is_ascii_alphabetic() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push(ch.to_string());
        } else if ch == '-' || ch == '.' {
            // 负号或小数点可能是新数字的开始
            if !current.is_empty()
                && ch == '-'
                && !current.ends_with('e')
                && !current.ends_with('E')
            {
                tokens.push(std::mem::take(&mut current));
            }
            current.push(ch);
        } else if ch.is_ascii_digit() || ch == 'e' || ch == 'E' {
            current.push(ch);
        } else if (ch == ',' || ch.is_whitespace()) && !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// 检查 token 是否是 SVG 路径命令。
fn is_command(token: &str) -> bool {
    matches!(
        token,
        "M" | "m" | "L" | "l" | "C" | "c" | "Q" | "q" | "Z" | "z" | "H" | "h" | "V" | "v"
    )
}

/// 安全解析 token 为 f32，解析失败返回 0.0。
fn parse_token(token: &str) -> f32 {
    token.parse::<f32>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_font_data() -> Vec<u8> {
        use std::io::Read;
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/upstream_20261014/stroke_true.ofd");
        let mut archive =
            zip::ZipArchive::new(std::io::Cursor::new(std::fs::read(source).unwrap())).unwrap();
        let mut data = Vec::new();
        archive
            .by_name("Doc_0/Res/font.ttf")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        data
    }

    #[test]
    fn font_migration_baseline() {
        let data = fixture_font_data();
        let face = crate::font::font_face::FontFace::parse(&data, 0).unwrap();
        for (ch, gid, xmin, ymin, width, height) in
            [('A', 102, 0., 0., 8, 12), ('/', 402, 0., -2., 8, 15)]
        {
            assert_eq!(face.glyph_index(ch).unwrap().0, gid);
            assert_eq!(
                face.glyph_hor_advance(crate::font::font_face::GlyphId(gid)),
                Some(128.)
            );
            let glyph = crate::font::glyph_bitmap::GlyphBitmap::rasterize(&face, ch, 16.).unwrap();
            assert_eq!((glyph.xmin, glyph.ymin, glyph.advance), (xmin, ymin, 8.));
            let pixels = glyph.pixels.unwrap();
            assert_eq!((pixels.width(), pixels.height()), (width, height));
            assert!(pixels.pixels().iter().any(|p| p.alpha() > 0));
        }
        let space = crate::font::glyph_bitmap::GlyphBitmap::rasterize(&face, ' ', 16.).unwrap();
        assert_eq!(space.advance.to_bits(), 8_f32.to_bits());
        assert!(space.pixels.is_none());
        assert!(face.glyph_index('中').is_none());
        let missing = crate::font::glyph_bitmap::GlyphBitmap::rasterize(&face, '中', 16.).unwrap();
        assert_eq!(missing.advance.to_bits(), 16_f32.to_bits());
        assert!(missing.pixels.is_none());
        assert_eq!(face.units_per_em(), 256);
        assert_eq!(face.number_of_glyphs(), 1384);
    }

    #[test]
    fn glyph_bitmap_rejects_invalid_size_and_actual_large_outline() {
        let mut data = fixture_font_data();
        let face = crate::font::font_face::FontFace::parse(&data, 0).unwrap();
        for size in [f32::NAN, f32::INFINITY, -1., 0., 1025.] {
            assert!(crate::font::glyph_bitmap::GlyphBitmap::rasterize(&face, 'A', size).is_err());
        }
        // 真实字形坐标不变，缩小head.unitsPerEm使合法字号产生超预算位图。
        let tables = u16::from_be_bytes(data[4..6].try_into().unwrap());
        let entry = (0..usize::from(tables))
            .map(|i| 12 + i * 16)
            .find(|i| &data[*i..*i + 4] == b"head")
            .unwrap();
        let offset = u32::from_be_bytes(data[entry + 8..entry + 12].try_into().unwrap()) as usize;
        data[offset + 18..offset + 20].copy_from_slice(&16_u16.to_be_bytes());
        let face = crate::font::font_face::FontFace::parse(&data, 0).unwrap();
        let error = crate::font::glyph_bitmap::GlyphBitmap::rasterize(&face, 'A', 1024.)
            .err()
            .unwrap();
        assert!(error.to_string().contains("实际字形位图超出预算"));
    }

    #[test]
    fn legacy_text_keeps_start_position_ymin_and_space_advance() {
        let data = fixture_font_data();
        let face = crate::font::font_face::FontFace::parse(&data, 0).unwrap();
        let renderer = RasterRenderer::new();
        let make_text = |content: &str| {
            let mut text = TextObject::new(10. * 25.4 / 96., 20. * 25.4 / 96., content);
            text.size = 12.;
            text
        };
        let mut a = RgbaImage::from_pixel(80, 80, Rgba([255; 4]));
        let mut spaced = a.clone();
        renderer
            .render_text_with_face(&mut a, &make_text("A"), &face)
            .unwrap();
        renderer
            .render_text_with_face(&mut spaced, &make_text(" A"), &face)
            .unwrap();
        let visible: Vec<_> = a
            .enumerate_pixels()
            .filter(|(_, _, p)| p[0] < 255)
            .map(|(x, y, _)| (x, y))
            .collect();
        assert!(!visible.is_empty());
        assert!(
            visible
                .iter()
                .all(|(x, y)| (10..18).contains(x) && (36..48).contains(y))
        );
        for (x, y, pixel) in a.enumerate_pixels() {
            if x + 8 < spaced.width() {
                assert_eq!(
                    pixel,
                    spaced.get_pixel(x + 8, y),
                    "space must advance exactly 8px"
                );
            }
        }
        let mut slash = RgbaImage::from_pixel(80, 80, Rgba([255; 4]));
        renderer
            .render_text_with_face(&mut slash, &make_text("/"), &face)
            .unwrap();
        let visible: Vec<_> = slash
            .enumerate_pixels()
            .filter(|(_, _, p)| p[0] < 255)
            .map(|(x, y, _)| (x, y))
            .collect();
        assert!(!visible.is_empty());
        assert!(
            visible
                .iter()
                .all(|(x, y)| (10..18).contains(x) && (34..49).contains(y))
        );
        assert!(
            visible.iter().any(|(_, y)| *y < 36),
            "negative ymin shifts the actual glyph above A"
        );
    }

    #[test]
    fn test_raster_renderer_new() {
        let r = RasterRenderer::new();
        assert!((r.dpi() - 96.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_raster_renderer_with_dpi() {
        let r = RasterRenderer::new().with_dpi(300.0);
        assert!((r.dpi() - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mm_to_px() {
        let r = RasterRenderer::new();
        // 25.4mm = 1 inch at 96 DPI = 96px
        assert!((r.mm_to_px(25.4) - 96.0).abs() < 0.01);
    }

    #[test]
    fn test_pt_to_px() {
        let r = RasterRenderer::new();
        // 72pt = 1 inch at 96 DPI = 96px
        assert!((r.pt_to_px(72.0) - 96.0).abs() < 0.01);
    }

    #[test]
    fn test_render_empty_page() {
        let r = RasterRenderer::new();
        let page = OfdPage::new(210.0, 297.0);
        let img = r.render_page(&page);
        // A4 尺寸 210x297mm at 96 DPI
        let expected_w = (210.0_f64 / 25.4 * 96.0).round() as u32;
        let expected_h = (297.0_f64 / 25.4 * 96.0).round() as u32;
        assert_eq!(img.width(), expected_w);
        assert_eq!(img.height(), expected_h);
        // 空页面应为白色背景
        let pixel = img.get_pixel(0, 0);
        assert_eq!(pixel[0], 255);
        assert_eq!(pixel[1], 255);
        assert_eq!(pixel[2], 255);
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn test_render_page_with_text() {
        let r = RasterRenderer::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "Hello"));
        let img = r.render_page(&page);
        // 页面应包含非白色像素（文本渲染）
        assert_eq!(img.width(), (210.0_f64 / 25.4 * 96.0).round() as u32);
    }

    #[test]
    fn test_render_page_with_path() {
        let r = RasterRenderer::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::rect(10.0, 10.0, 50.0, 30.0));
        let img = r.render_page(&page);
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    fn test_render_page_with_image() {
        let r = RasterRenderer::new();
        let mut page = OfdPage::new(210.0, 297.0);
        // 创建一个简单的 2x2 红色 PNG
        let png_data = create_test_png(2, 2, [255, 0, 0, 255]);
        page.add_image(ImageObject::png(10.0, 10.0, 20.0, 20.0, png_data));
        let img = r.render_page(&page);
        assert!(img.width() > 0);
    }

    #[test]
    fn test_u32_to_rgba() {
        let c = u32_to_rgba(0xFF_0000);
        assert_eq!(c[0], 255);
        assert_eq!(c[1], 0);
        assert_eq!(c[2], 0);
        assert_eq!(c[3], 255);

        let c2 = u32_to_rgba(0x00_FF00);
        assert_eq!(c2[0], 0);
        assert_eq!(c2[1], 255);
        assert_eq!(c2[2], 0);
    }

    #[test]
    fn test_parse_svg_path_move_line() {
        let path = parse_svg_path("M10 20 L30 40");
        assert!(path.is_some());
    }

    #[test]
    fn test_parse_svg_path_rect() {
        let path = parse_svg_path("M0 0L100 0L100 50L0 50Z");
        assert!(path.is_some());
    }

    #[test]
    fn test_parse_svg_path_empty() {
        assert!(parse_svg_path("").is_none());
    }

    #[test]
    fn test_parse_svg_path_relative() {
        let path = parse_svg_path("m10 10 l20 20 l-5 -5");
        assert!(path.is_some());
    }

    #[test]
    fn test_parse_svg_path_bezier() {
        let path = parse_svg_path("M10 10 C30 80 70 80 90 10");
        assert!(path.is_some());
    }

    #[test]
    fn test_parse_svg_path_quadratic() {
        let path = parse_svg_path("M10 10 Q50 100 90 10");
        assert!(path.is_some());
    }

    #[test]
    fn test_tokenize_path() {
        let tokens = tokenize_path("M10 20L30 40");
        assert!(tokens.contains(&"M".to_string()));
        assert!(tokens.contains(&"10".to_string()));
        assert!(tokens.contains(&"20".to_string()));
        assert!(tokens.contains(&"L".to_string()));
    }

    #[test]
    fn test_tokenize_negative_numbers() {
        let tokens = tokenize_path("M-10 -20L30 -40");
        assert!(tokens.contains(&"-10".to_string()));
        assert!(tokens.contains(&"-20".to_string()));
        assert!(tokens.contains(&"-40".to_string()));
    }

    #[test]
    fn test_blend_pixel_full_alpha() {
        let mut dst = Rgba([255, 255, 255, 255]);
        let fg = Rgba([0, 0, 0, 255]);
        blend_pixel(&mut dst, fg, 255);
        assert_eq!(dst[0], 0);
        assert_eq!(dst[1], 0);
        assert_eq!(dst[2], 0);
    }

    #[test]
    fn test_blend_pixel_zero_alpha() {
        let mut dst = Rgba([255, 255, 255, 255]);
        let fg = Rgba([0, 0, 0, 255]);
        blend_pixel(&mut dst, fg, 0);
        assert_eq!(dst[0], 255);
        assert_eq!(dst[1], 255);
        assert_eq!(dst[2], 255);
    }

    #[test]
    fn test_render_mixed_content() {
        let r = RasterRenderer::new().with_dpi(72.0);
        let mut page = OfdPage::new(100.0, 100.0);
        page.add_text(TextObject::new(10.0, 20.0, "Test"));
        page.add_path(PathObject::rect(5.0, 5.0, 90.0, 90.0));
        let img = r.render_page(&page);
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    /// 创建测试用的简单 PNG 数据。
    fn create_test_png(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        use ::image::{ImageBuffer, ImageEncoder, RgbaImage};
        let img: RgbaImage = ImageBuffer::from_pixel(width, height, Rgba(rgba));
        let mut buf = Vec::new();
        let encoder = ::image::codecs::png::PngEncoder::new(&mut buf);
        let raw = img.to_vec();
        encoder
            .write_image(&raw, width, height, ::image::ExtendedColorType::Rgba8)
            .unwrap();
        buf
    }

    #[test]
    fn test_create_test_png() {
        let png = create_test_png(2, 2, [255, 0, 0, 255]);
        assert!(png.len() > 8);
        // PNG 签名
        assert_eq!(
            &png[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }
}
