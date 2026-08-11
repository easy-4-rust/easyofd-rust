//! 裁剪区域工厂。
//!
//! 对应 Java 版 `ofdrw-graphics2d` 中的裁剪区域构建工具，
//! 用于创建 [`CT_Clip`] 裁剪区域，支持矩形、圆形、椭圆、多边形等常见形状。

use easyofd_core::page_description::clips::{CT_Clip, ClipPath};

/// 裁剪区域工厂。
///
/// 提供常见裁剪形状的快速构建方法，每个方法返回一个 [`CT_Clip`]。
#[derive(Debug, Clone, Copy)]
pub struct ClipFactory;

impl ClipFactory {
    /// 创建矩形裁剪区域。
    ///
    /// # 参数
    /// - `x`, `y`：左上角坐标（mm）
    /// - `w`, `h`：宽高（mm）
    #[must_use]
    pub fn rect(x: f64, y: f64, w: f64, h: f64) -> CT_Clip {
        let data = format!("M{x} {y}L{} {y}L{} {}L{x} {}Z", x + w, x + w, y + h, y + h);
        let mut clip = CT_Clip::new();
        clip.add_path(ClipPath::new(&data));
        clip
    }

    /// 创建圆形裁剪区域。
    ///
    /// # 参数
    /// - `cx`, `cy`：圆心坐标（mm）
    /// - `r`：半径（mm）
    #[must_use]
    pub fn circle(cx: f64, cy: f64, r: f64) -> CT_Clip {
        Self::ellipse(cx, cy, r, r)
    }

    /// 创建椭圆裁剪区域。
    ///
    /// # 参数
    /// - `cx`, `cy`：中心坐标（mm）
    /// - `rx`, `ry`：x/y 方向半径（mm）
    #[must_use]
    pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64) -> CT_Clip {
        let kx = 0.552_284_749_8 * rx;
        let ky = 0.552_284_749_8 * ry;
        let data = format!(
            "M{cx} {top}\
             C{r_cx} {top} {right} {r_cy} {right} {cy}\
             C{right} {b_cy} {r_cx} {bottom} {cx} {bottom}\
             C{l_cx} {bottom} {left} {b_cy} {left} {cy}\
             C{left} {r_cy} {l_cx} {top} {cx} {top}Z",
            top = cy - ry,
            bottom = cy + ry,
            left = cx - rx,
            right = cx + rx,
            r_cx = cx + kx,
            l_cx = cx - kx,
            r_cy = cy + ky,
            b_cy = cy - ky,
        );
        let mut clip = CT_Clip::new();
        clip.add_path(ClipPath::new(&data));
        clip
    }

    /// 创建多边形裁剪区域。
    ///
    /// # 参数
    /// - `points`：顶点坐标列表 `[(x, y), ...]`
    #[must_use]
    pub fn polygon(points: &[(f64, f64)]) -> CT_Clip {
        if points.is_empty() {
            return CT_Clip::new();
        }
        let mut data = format!("M{} {}", points[0].0, points[0].1);
        for &(x, y) in &points[1..] {
            let _ = std::fmt::Write::write_fmt(&mut data, format_args!("L{x} {y}"));
        }
        data.push('Z');
        let mut clip = CT_Clip::new();
        clip.add_path(ClipPath::new(&data));
        clip
    }

    /// 创建带变换矩阵的裁剪区域。
    ///
    /// # 参数
    /// - `data`：路径数据字符串
    /// - `transform`：变换矩阵参数 `(a, b, c, d, e, f)`
    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn with_transform(data: &str, transform: (f64, f64, f64, f64, f64, f64)) -> CT_Clip {
        let (a, b, c, d, e, f) = transform;
        let mut path = ClipPath::new(data);
        path.set_transform(easyofd_core::basic_type::ST_Array::transform(
            a, b, c, d, e, f,
        ));
        let mut clip = CT_Clip::new();
        clip.add_path(path);
        clip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_clip() {
        let clip = ClipFactory::rect(0.0, 0.0, 100.0, 50.0);
        assert_eq!(clip.paths().len(), 1);
        let data = clip.paths()[0].data();
        assert!(data.starts_with('M'));
        assert!(data.ends_with('Z'));
        assert!(data.contains("L100"));
    }

    #[test]
    fn test_circle_clip() {
        let clip = ClipFactory::circle(50.0, 50.0, 25.0);
        assert_eq!(clip.paths().len(), 1);
        let data = clip.paths()[0].data();
        assert!(data.contains('C'));
        assert!(data.ends_with('Z'));
    }

    #[test]
    fn test_ellipse_clip() {
        let clip = ClipFactory::ellipse(50.0, 50.0, 30.0, 15.0);
        assert_eq!(clip.paths().len(), 1);
        let data = clip.paths()[0].data();
        assert_eq!(data.matches('C').count(), 4);
    }

    #[test]
    fn test_polygon_clip() {
        let points = [(0.0, 0.0), (100.0, 0.0), (50.0, 80.0)];
        let clip = ClipFactory::polygon(&points);
        assert_eq!(clip.paths().len(), 1);
        let data = clip.paths()[0].data();
        assert!(data.starts_with('M'));
        assert!(data.ends_with('Z'));
    }

    #[test]
    fn test_empty_polygon() {
        let clip = ClipFactory::polygon(&[]);
        assert!(clip.paths().is_empty());
    }

    #[test]
    fn test_with_transform() {
        let clip =
            ClipFactory::with_transform("M0 0L100 0L100 100Z", (1.0, 0.0, 0.0, 1.0, 10.0, 20.0));
        assert_eq!(clip.paths().len(), 1);
        assert!(clip.paths()[0].transform().is_some());
    }

    #[test]
    fn test_rect_clip_dimensions() {
        let clip = ClipFactory::rect(10.0, 20.0, 30.0, 40.0);
        let data = clip.paths()[0].data();
        assert!(data.contains("M10"));
        assert!(data.contains("20"));
        assert!(data.contains("40"));
        assert!(data.contains("60"));
    }
}
