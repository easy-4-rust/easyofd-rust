//! 点坐标计算工具。
//!
//! 对应 Java: org.ofdrw.converter.utils.PointUtil
//!
//! 提供 CTM 变换、路径点坐标校准、文本坐标计算等纯数学函数。
//! 依赖 OFD 核心类型的函数（如 ST_Box、TextCode 等）暂不移植，
//! 因为它们需要 easyofd-core 中对应的类型支持。

use crate::point::PathPoint;
use crate::utils::common_util::converter_dpi;

/// 早期 Foxit OFD 文件使用 300 DPI 设备坐标存储路径坐标。
const LEGACY_PATH_DPI: f64 = 300.0;
/// 300 DPI 到毫米的换算因子。
const LEGACY_PATH_MM_SCALE: f64 = 25.4 / LEGACY_PATH_DPI;

/// CTM 坐标变换。
///
/// 对应 Java `PointUtil.ctmCalPoint(x, y, ctm)`。
///
/// 将点 (x, y) 通过 6 元素 CTM 数组变换到新坐标：
/// ```text
/// x' = x * ctm[0] + y * ctm[2] + ctm[4]
/// y' = x * ctm[1] + y * ctm[3] + ctm[5]
/// ```
pub fn ctm_transform_point(x: f64, y: f64, ctm: &[f64; 6]) -> [f64; 2] {
    let ctm_x = x * ctm[0] + y * ctm[2] + ctm[4];
    let ctm_y = x * ctm[1] + y * ctm[3] + ctm[5];
    [ctm_x, ctm_y]
}

/// CTM 向量变换（不含平移分量）。
///
/// 与 `ctm_transform_point` 的区别在于不加 `ctm[4]` 和 `ctm[5]` 平移量。
pub fn ctm_transform_vector(dx: f64, dy: f64, ctm: &[f64; 6]) -> [f64; 2] {
    let ctm_x = dx * ctm[0] + dy * ctm[2];
    let ctm_y = dx * ctm[1] + dy * ctm[3];
    [ctm_x, ctm_y]
}

/// 校准路径点坐标（毫米→72 DPI 点）。
///
/// 对应 Java `PointUtil.calPathPoint(abbreviatedPoint)`。
///
/// 将路径点列表中的原始毫米坐标转换为 72 DPI 下的点坐标。
pub fn cal_path_points(points: &[PathPoint]) -> Vec<PathPoint> {
    points
        .iter()
        .map(|p| match p.point_type.as_str() {
            "M" | "L" | "C" | "S" => {
                let mut np = p.clone();
                np.x1 = converter_dpi(p.x1 as f64) as f32;
                np.y1 = converter_dpi(p.y1 as f64) as f32;
                np
            }
            "B" => {
                let mut np = p.clone();
                np.x1 = converter_dpi(p.x1 as f64) as f32;
                np.y1 = converter_dpi(p.y1 as f64) as f32;
                np.x2 = converter_dpi(p.x2 as f64) as f32;
                np.y2 = converter_dpi(p.y2 as f64) as f32;
                np.x3 = converter_dpi(p.x3 as f64) as f32;
                np.y3 = converter_dpi(p.y3 as f64) as f32;
                np
            }
            "Q" => {
                let mut np = p.clone();
                np.x1 = converter_dpi(p.x1 as f64) as f32;
                np.y1 = converter_dpi(p.y1 as f64) as f32;
                np.x2 = converter_dpi(p.x2 as f64) as f32;
                np.y2 = converter_dpi(p.y2 as f64) as f32;
                np
            }
            "A" => {
                let mut np = p.clone();
                np.rx = converter_dpi(p.rx as f64) as f32;
                np.ry = converter_dpi(p.ry as f64) as f32;
                np.x = converter_dpi(p.x as f64) as f32;
                np.y = converter_dpi(p.y as f64) as f32;
                np
            }
            _ => p.clone(),
        })
        .collect()
}

/// 位置调整：将相对坐标加上 boundary 偏移。
///
/// 对应 Java `PointUtil.adjustPos(width, height, x, y, boundary)`。
///
/// # 参数
/// - `x, y`：相对坐标
/// - `boundary`：可选的外接矩形 `(top_left_x, top_left_y, width, height)`
///
/// # 返回
/// 调整后的绝对坐标 `[x, y]`。
pub fn adjust_pos(x: f64, y: f64, boundary: Option<&[f64; 4]>) -> [f64; 2] {
    match boundary {
        Some(b) => [b[0] + x, b[1] + y],
        None => [x, y],
    }
}

/// 检测是否为早期 Foxit 绝对路径坐标。
///
/// 对应 Java `PointUtil.isLegacyAbsolutePath(...)`。
///
/// 早期 Foxit OFD 文件将路径坐标存储为 300 DPI 设备坐标，
/// 而非标准的毫米坐标。此函数通过比较坐标范围来检测这种情况。
pub fn is_legacy_absolute_path(
    width: f64,
    height: f64,
    boundary: Option<&[f64; 4]>,
    points: &[PathPoint],
    has_ctm: bool,
    ctm: Option<&[f64; 6]>,
) -> bool {
    let boundary = match boundary {
        Some(b) => b,
        None => return false,
    };
    if !has_ctm || ctm.is_none() || points.is_empty() {
        return false;
    }
    let ctm = ctm.unwrap();

    // 原始坐标范围
    let raw = path_bounds(points, None, 1.0);
    if raw.is_none() {
        return false;
    }
    let (raw_max_x, raw_max_y) = raw.unwrap();
    if raw_max_x <= width * 2.0 && raw_max_y <= height * 2.0 {
        return false;
    }

    // 标准 CTM 变换后的坐标范围
    let standard = path_bounds(points, Some(ctm), 1.0);
    if let Some((std_max_x, std_max_y)) = standard
        && std_max_x >= 0.0
        && std_max_x <= boundary[2]
        && std_max_y >= 0.0
        && std_max_y <= boundary[3]
    {
        return false;
    }

    // 300 DPI 换算后的坐标范围
    let legacy = path_bounds(points, None, LEGACY_PATH_MM_SCALE);
    let tolerance = 0.5_f64.max(boundary[2].max(boundary[3]) * 0.1);
    if let Some((leg_max_x, leg_max_y)) = legacy {
        leg_max_x >= boundary[0] - tolerance
            && leg_max_x <= boundary[0] + boundary[2] + tolerance
            && leg_max_y >= boundary[1] - tolerance
            && leg_max_y <= boundary[1] + boundary[3] + tolerance
    } else {
        false
    }
}

/// 计算路径点的坐标范围（最大绝对值）。
fn path_bounds(points: &[PathPoint], ctm: Option<&[f64; 6]>, scale: f64) -> Option<(f64, f64)> {
    let mut max_abs_x = 0.0_f64;
    let mut max_abs_y = 0.0_f64;
    let mut has_point = false;

    for point in points {
        let coords: Vec<(f64, f64)> = match point.point_type.as_str() {
            "M" | "L" | "S" => vec![(point.x1 as f64, point.y1 as f64)],
            "B" => vec![
                (point.x1 as f64, point.y1 as f64),
                (point.x2 as f64, point.y2 as f64),
                (point.x3 as f64, point.y3 as f64),
            ],
            "Q" => vec![
                (point.x1 as f64, point.y1 as f64),
                (point.x2 as f64, point.y2 as f64),
            ],
            "A" => vec![(point.x as f64, point.y as f64)],
            _ => continue,
        };

        for (x, y) in coords {
            let (mut tx, mut ty) = if let Some(ctm) = ctm {
                let t = ctm_transform_point(x, y, ctm);
                (t[0], t[1])
            } else {
                (x, y)
            };
            tx *= scale;
            ty *= scale;
            max_abs_x = max_abs_x.max(tx.abs());
            max_abs_y = max_abs_y.max(ty.abs());
            has_point = true;
        }
    }

    if has_point {
        Some((max_abs_x, max_abs_y))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_point(t: &str, x1: f32, y1: f32) -> PathPoint {
        PathPoint::new(t, x1, y1)
    }

    #[test]
    fn test_ctm_transform_point_identity() {
        let ctm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let [x, y] = ctm_transform_point(5.0, 10.0, &ctm);
        assert!((x - 5.0).abs() < 1e-10);
        assert!((y - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_ctm_transform_point_translate() {
        let ctm = [1.0, 0.0, 0.0, 1.0, 100.0, 200.0];
        let [x, y] = ctm_transform_point(5.0, 10.0, &ctm);
        assert!((x - 105.0).abs() < 1e-10);
        assert!((y - 210.0).abs() < 1e-10);
    }

    #[test]
    fn test_ctm_transform_point_scale() {
        let ctm = [2.0, 0.0, 0.0, 3.0, 0.0, 0.0];
        let [x, y] = ctm_transform_point(5.0, 10.0, &ctm);
        assert!((x - 10.0).abs() < 1e-10);
        assert!((y - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_ctm_transform_vector_no_translate() {
        let ctm = [1.0, 0.0, 0.0, 1.0, 100.0, 200.0];
        let [x, y] = ctm_transform_vector(5.0, 10.0, &ctm);
        assert!((x - 5.0).abs() < 1e-10);
        assert!((y - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_cal_path_points_move() {
        let points = vec![make_point("M", 10.0, 20.0)];
        let result = cal_path_points(&points);
        assert_eq!(result.len(), 1);
        let expected_x = converter_dpi(10.0) as f32;
        let expected_y = converter_dpi(20.0) as f32;
        assert!((result[0].x1 - expected_x).abs() < 0.01);
        assert!((result[0].y1 - expected_y).abs() < 0.01);
    }

    #[test]
    fn test_cal_path_points_bezier() {
        let points = vec![PathPoint::bezier(1.0, 2.0, 3.0, 4.0, 5.0, 6.0)];
        let result = cal_path_points(&points);
        assert_eq!(result.len(), 1);
        let expected_x1 = converter_dpi(1.0) as f32;
        assert!((result[0].x1 - expected_x1).abs() < 0.01);
    }

    #[test]
    fn test_cal_path_points_close() {
        let points = vec![PathPoint::close()];
        let result = cal_path_points(&points);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].point_type, "C");
    }

    #[test]
    fn test_adjust_pos_no_boundary() {
        let [x, y] = adjust_pos(10.0, 20.0, None);
        assert!((x - 10.0).abs() < 1e-10);
        assert!((y - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_adjust_pos_with_boundary() {
        let boundary = [5.0, 10.0, 100.0, 200.0];
        let [x, y] = adjust_pos(3.0, 7.0, Some(&boundary));
        assert!((x - 8.0).abs() < 1e-10);
        assert!((y - 17.0).abs() < 1e-10);
    }

    #[test]
    fn test_is_legacy_absolute_path_no_boundary() {
        assert!(!is_legacy_absolute_path(
            100.0,
            100.0,
            None,
            &[],
            false,
            None
        ));
    }

    #[test]
    fn test_is_legacy_absolute_path_empty_points() {
        let boundary = [0.0, 0.0, 100.0, 100.0];
        assert!(!is_legacy_absolute_path(
            100.0,
            100.0,
            Some(&boundary),
            &[],
            true,
            Some(&[1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
        ));
    }

    #[test]
    fn test_path_bounds_empty() {
        assert!(path_bounds(&[], None, 1.0).is_none());
    }

    #[test]
    fn test_path_bounds_single_point() {
        let points = vec![make_point("M", 10.0, 20.0)];
        let bounds = path_bounds(&points, None, 1.0).unwrap();
        assert!((bounds.0 - 10.0).abs() < 1e-10);
        assert!((bounds.1 - 20.0).abs() < 1e-10);
    }
}
