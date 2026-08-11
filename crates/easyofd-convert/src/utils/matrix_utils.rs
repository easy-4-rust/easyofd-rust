//! 矩阵工具。
//!
//! 对应 Java: org.ofdrw.converter.utils.MatrixUtils

/// 2D 仿射变换矩阵工具。
///
/// 对应 Java: org.ofdrw.converter.utils.MatrixUtils
///
/// 提供 2D 仿射变换矩阵的创建和运算功能。
/// 矩阵以 3x2 形式表示: [a, b, c, d, e, f]，
/// 对应变换:
/// ```text
/// | a  b  0 |
/// | c  d  0 |
/// | e  f  1 |
/// ```
pub struct MatrixUtils;

impl MatrixUtils {
    /// 创建单位矩阵 [1, 0, 0, 1, 0, 0]。
    #[must_use]
    pub fn identity() -> [f64; 6] {
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    }

    /// 创建平移矩阵。
    #[must_use]
    pub fn translate(tx: f64, ty: f64) -> [f64; 6] {
        [1.0, 0.0, 0.0, 1.0, tx, ty]
    }

    /// 创建缩放矩阵。
    #[must_use]
    pub fn scale(sx: f64, sy: f64) -> [f64; 6] {
        [sx, 0.0, 0.0, sy, 0.0, 0.0]
    }

    /// 创建旋转矩阵（角度，顺时针）。
    #[must_use]
    pub fn rotate(angle_deg: f64) -> [f64; 6] {
        let rad = angle_deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        [cos, sin, -sin, cos, 0.0, 0.0]
    }

    /// 矩阵乘法：m1 * m2。
    #[must_use]
    pub fn multiply(m1: &[f64; 6], m2: &[f64; 6]) -> [f64; 6] {
        [
            m1[0] * m2[0] + m1[1] * m2[2],
            m1[0] * m2[1] + m1[1] * m2[3],
            m1[2] * m2[0] + m1[3] * m2[2],
            m1[2] * m2[1] + m1[3] * m2[3],
            m1[4] * m2[0] + m1[5] * m2[2] + m2[4],
            m1[4] * m2[1] + m1[5] * m2[3] + m2[5],
        ]
    }

    /// 对点 (x, y) 应用变换矩阵。
    #[must_use]
    pub fn transform_point(m: &[f64; 6], x: f64, y: f64) -> (f64, f64) {
        (
            m[0] * x + m[2] * y + m[4],
            m[1] * x + m[3] * y + m[5],
        )
    }

    /// 解析 "a b c d e f" 格式的变换矩阵字符串。
    pub fn parse(s: &str) -> Result<[f64; 6], String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(format!("变换矩阵需要 6 个值，得到 {}", parts.len()));
        }
        let mut m = [0.0; 6];
        for (i, part) in parts.iter().enumerate() {
            m[i] = part
                .parse::<f64>()
                .map_err(|e| format!("解析变换矩阵第 {} 个值失败: {e}", i + 1))?;
        }
        Ok(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = MatrixUtils::identity();
        assert_eq!(m, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_translate() {
        let m = MatrixUtils::translate(10.0, 20.0);
        assert_eq!(m[4], 10.0);
        assert_eq!(m[5], 20.0);
    }

    #[test]
    fn test_scale() {
        let m = MatrixUtils::scale(2.0, 3.0);
        assert_eq!(m[0], 2.0);
        assert_eq!(m[3], 3.0);
    }

    #[test]
    fn test_rotate() {
        let m = MatrixUtils::rotate(90.0);
        // 90度旋转: cos(90)=0, sin(90)=1
        assert!((m[0] - 0.0).abs() < 1e-10);
        assert!((m[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_multiply_identity() {
        let m = MatrixUtils::translate(10.0, 20.0);
        let identity = MatrixUtils::identity();
        let result = MatrixUtils::multiply(&m, &identity);
        assert!((result[4] - 10.0).abs() < 1e-10);
        assert!((result[5] - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_transform_point() {
        let m = MatrixUtils::translate(5.0, 10.0);
        let (x, y) = MatrixUtils::transform_point(&m, 1.0, 2.0);
        assert!((x - 6.0).abs() < 1e-10);
        assert!((y - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_parse() {
        let m = MatrixUtils::parse("1 0 0 1 10 20").unwrap();
        assert_eq!(m, [1.0, 0.0, 0.0, 1.0, 10.0, 20.0]);
    }

    #[test]
    fn test_parse_invalid() {
        assert!(MatrixUtils::parse("1 0 0").is_err());
        assert!(MatrixUtils::parse("a b c d e f").is_err());
    }
}
