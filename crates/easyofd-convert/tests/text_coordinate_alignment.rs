//! 文字定位与CTM的行为回归。
use easyofd_convert::utils::text_layout::layout_text_run;
use easyofd_core::{CT_Text, TextCode};
use easyofd_reader::TextRenderRun;
fn run(ctm: [f64; 6], boundary: &str, code: TextCode) -> TextRenderRun {
    let mut text = CT_Text::new(1, boundary);
    text.text_codes.push(code);
    TextRenderRun {
        content_index: 0,
        text,
        ctm: Some(ctm),
        line_width_mm: 0.3,
        font_name: None,
        font_data: None,
    }
}
#[test]
fn translation_applies_only_to_origin() {
    let r = run(
        [2., 0., 0., 3., 100., 200.],
        "10 20 200 300",
        TextCode::with_content("AB")
            .coordinate(1., 2.)
            .delta_x(vec![4.])
            .delta_y(vec![5.]),
    );
    let p = layout_text_run(&r, &[1., 1.]).unwrap();
    assert_point(p[0].origin_mm, [112., 226.]);
    assert_point(p[1].origin_mm, [120., 241.]);
}
#[test]
fn vertical_step_does_not_leak_translation() {
    let r = run(
        [0., 1., -1., 0., 0.5, -2.],
        "207.5 30 20 20",
        TextCode::with_content("下载次数：1")
            .coordinate(1.5, 0.)
            .delta_x(vec![3.18; 5]),
    );
    let p = layout_text_run(&r, &[3.18; 6]).unwrap();
    assert_point(p[0].origin_mm, [208., 29.5]);
    for pair in p.windows(2) {
        assert!((pair[1].origin_mm[1] - pair[0].origin_mm[1] - 3.18).abs() < 1e-9);
        assert!((pair[1].origin_mm[0] - 208.).abs() < 1e-9);
    }
}
#[test]
fn rejects_bad_metrics_and_glyph_mapping() {
    let mut r = run(
        [1., 0., 0., 1., 0., 0.],
        "0 0 20 20",
        TextCode::with_content("AB"),
    );
    assert!(layout_text_run(&r, &[f64::NAN, 1.]).is_err());
    assert!(layout_text_run(&r, &[]).is_err());
    r.text.cg_transforms.push(
        easyofd_core::CT_CGTransform::new()
            .code_position(3)
            .code_count(1)
            .glyph_count(1)
            .glyphs(vec![1]),
    );
    assert!(layout_text_run(&r, &[1., 1.]).is_err());
}

#[test]
fn omitted_coordinates_continue_each_axis_without_losing_segments() {
    let mut r = run(
        [1., 0., 0., 1., 0., 0.],
        "10 20 200 300",
        TextCode::with_content("AB")
            .coordinate(1., 2.)
            .delta_x(vec![4.])
            .delta_y(vec![5.]),
    );
    r.text
        .text_codes
        .push(TextCode::with_content("CD").y(20.).delta_x(vec![2.]));
    let p = layout_text_run(&r, &[1.; 4]).unwrap();
    assert_eq!(p.iter().map(|g| g.character).collect::<String>(), "ABCD");
    assert_point(p[0].origin_mm, [11., 22.]);
    assert_point(p[1].origin_mm, [15., 27.]);
    assert_point(p[2].origin_mm, [19., 40.]);
    assert_point(p[3].origin_mm, [21., 40.]);
}

fn assert_point(actual: [f64; 2], expected: [f64; 2]) {
    for (a, e) in actual.into_iter().zip(expected) {
        assert!((a - e).abs() < 1e-9, "{actual:?} != {expected:?}");
    }
}
