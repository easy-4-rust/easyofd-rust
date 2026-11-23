//! 验证文件转换入口在像素分配前拒绝危险尺寸。
use easyofd_convert::converter::AWTMaker;
#[test]
fn invalid_dpi_is_rejected_at_file_conversion_boundary() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream_20261014/stroke_true.ofd");
    for dpi in [0.0, f64::NAN] {
        let target =
            std::env::temp_dir().join(format!("raster-budget-{}-{dpi}.png", std::process::id()));
        let result = AWTMaker::from_file(&source)
            .unwrap()
            .with_dpi(dpi)
            .convert(&target);
        let _ = std::fs::remove_file(target);
        assert!(result.is_err(), "非法DPI不能返回成功: {dpi}");
    }
}
#[test]
fn dangerous_dimensions_are_rejected_before_allocation() {
    use easyofd_convert::converter::RasterRenderer;
    use easyofd_core::OfdPage;
    let renderer = RasterRenderer::new();
    for (w, h) in [
        (f64::NAN, 297.),
        (f64::INFINITY, 297.),
        (1e12, 1e12),
        (-1., 297.),
        (0., 297.),
    ] {
        assert!(renderer.try_render_page(&OfdPage::new(w, h)).is_err());
    }
    assert!(
        renderer
            .clone()
            .with_dpi(f64::INFINITY)
            .try_render_page(&OfdPage::new(210., 297.))
            .is_err()
    );
    let image = renderer.try_render_page(&OfdPage::new(25.4, 25.4)).unwrap();
    assert_eq!(image.dimensions(), (96, 96));
}
#[test]
fn invalid_text_size_cannot_bypass_safe_entry() {
    use easyofd_convert::converter::RasterRenderer;
    use easyofd_core::{OfdPage, TextObject};
    let mut page = OfdPage::new(25.4, 25.4);
    let mut text = TextObject::new(0., 0., "A");
    text.size = 0.;
    page.add_text(text);
    assert!(RasterRenderer::new().try_render_page(&page).is_err());
}
#[test]
fn huge_text_size_is_rejected_on_small_canvas() {
    use easyofd_convert::converter::RasterRenderer;
    use easyofd_core::{OfdPage, TextObject};
    for size in [f64::NAN, f64::INFINITY, 1e12, -1.] {
        let mut page = OfdPage::new(25.4, 25.4);
        let mut text = TextObject::new(0., 0., "A");
        text.size = size;
        page.add_text(text);
        assert!(RasterRenderer::new().try_render_page(&page).is_err());
    }
}
