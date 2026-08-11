//! 3x3 矩阵工具。
//!
//! 对应 Java: org.ofdrw.converter.utils.MatrixUtils

use crate::point::Tuple2;

/// 3x3 仿射变换矩阵（行主序）。
///
/// 对应 Java `MatrixUtils` 中操作的 `org.ujmp.core.Matrix`（3x3）。
/// 用于 OFD 渲染中的坐标变换（平移、缩放、镜像、CTM 叠加）。
///
/// 内存布局：
/// ```text
/// | a  b  0 |     indices: [0][0]=a  [0][1]=b  [0][2]=0
/// | c  d  0 |              [1][0]=c  [1][1]=d  [1][2]=0
/// | tx ty 1 |              [2][0]=tx [2][1]=ty [2][2]=1
/// ```
///
/// 与 Java 版的映射关系：`create(a,b,c,d,tx,ty)` 对应 `MatrixUtils.create(d1,d2,d3,d4,d5,d6)`。
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix3x3 {
    /// 3x3 数据，行主序存储。
    data: [[f64; 3]; 3],
}

impl Matrix3x3 {
    /// 创建单位矩阵。
    ///
    /// 对应 Java `MatrixUtils.base()`。
    pub fn identity() -> Self {
        Self {
            data: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// 从仿射变换参数创建矩阵。
    ///
    /// 对应 Java `MatrixUtils.create(d1, d2, d3, d4, d5, d6)`。
    ///
    /// # 参数
    /// - `a, b`：第一行前两列（缩放/旋转）
    /// - `c, d`：第二行前两列（缩放/旋转）
    /// - `tx, ty`：第三行前两列（平移）
    pub fn new(a: f64, b: f64, c: f64, d: f64, tx: f64, ty: f64) -> Self {
        Self {
            data: [[a, b, 0.0], [c, d, 0.0], [tx, ty, 1.0]],
        }
    }

    /// 从 CTM 数组（6 个元素）创建矩阵。
    ///
    /// 对应 Java `MatrixUtils.ctm(ctm)`。
    pub fn from_ctm(ctm: &[f64; 6]) -> Self {
        Self::new(ctm[0], ctm[1], ctm[2], ctm[3], ctm[4], ctm[5])
    }

    /// 获取指定位置的值。
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row][col]
    }

    /// 设置指定位置的值。
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row][col] = value;
    }

    /// 矩阵乘法（self * rhs）。
    ///
    /// 对应 Java 中的 `matrix.mtimes(other)`。
    pub fn multiply(&self, rhs: &Matrix3x3) -> Matrix3x3 {
        let mut result = [[0.0f64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.data[i][0] * rhs.data[0][j]
                    + self.data[i][1] * rhs.data[1][j]
                    + self.data[i][2] * rhs.data[2][j];
            }
        }
        Matrix3x3 { data: result }
    }

    /// 缩放变换。
    ///
    /// 对应 Java `MatrixUtils.scale(matrix, x, y)`。
    pub fn scale(&self, x: f64, y: f64) -> Matrix3x3 {
        self.multiply(&Matrix3x3::new(x, 0.0, 0.0, y, 0.0, 0.0))
    }

    /// 平移变换。
    ///
    /// 对应 Java `MatrixUtils.move(matrix, x, y)`。
    pub fn translate(&self, x: f64, y: f64) -> Matrix3x3 {
        self.multiply(&Matrix3x3::new(1.0, 0.0, 0.0, 1.0, x, y))
    }

    /// 镜像变换。
    ///
    /// 对应 Java `MatrixUtils.imageMatrix(matrix, a, b, c)`。
    /// 关于直线 `aX + bY + c = 0` 做镜像。
    pub fn mirror(&self, a: f64, b: f64, c: f64) -> Matrix3x3 {
        let denom = a * a + b * b;
        if denom.abs() < f64::EPSILON {
            return self.clone();
        }
        let mut image = Matrix3x3::identity();
        image.data[0][0] = a * a - b * b;
        image.data[0][1] = 2.0 * a * b;
        image.data[1][0] = 2.0 * a * b;
        image.data[1][1] = -(a * a - b * b);
        image.data[2][0] = 2.0 * a * c;
        image.data[2][1] = 2.0 * b * c;
        image.data[2][2] = -(a * a + b * b);

        // image = image * (-1 / denom)
        for row in &mut image.data {
            for val in row.iter_mut() {
                *val *= -1.0 / denom;
            }
        }
        self.multiply(&image)
    }

    /// 点变换：将 (x, y) 通过矩阵变换到新坐标。
    ///
    /// 对应 Java `MatrixUtils.pointTransform(ctm, x, y)`。
    pub fn transform_point(&self, x: f64, y: f64) -> Tuple2<f64, f64> {
        let new_x = x * self.data[0][0] + y * self.data[1][0] + self.data[2][0];
        let new_y = x * self.data[0][1] + y * self.data[1][1] + self.data[2][1];
        Tuple2::new(new_x, new_y)
    }

    /// 计算外接矩形的左上角。
    ///
    /// 对应 Java `MatrixUtils.leftTop(matrix)`。
    pub fn left_top(&self) -> Tuple2<f64, f64> {
        let corners = [
            self.transform_point_raw(0.0, 0.0),
            self.transform_point_raw(1.0, 0.0),
            self.transform_point_raw(1.0, 1.0),
            self.transform_point_raw(0.0, 1.0),
        ];
        let mut min_x = corners[0].0;
        let mut min_y = corners[0].1;
        for (x, y) in &corners[1..] {
            min_x = min_x.min(*x);
            min_y = min_y.min(*y);
        }
        Tuple2::new(min_x, min_y)
    }

    /// 内部点变换（返回元组）。
    fn transform_point_raw(&self, x: f64, y: f64) -> (f64, f64) {
        let new_x = x * self.data[0][0] + y * self.data[1][0] + self.data[2][0];
        let new_y = x * self.data[0][1] + y * self.data[1][1] + self.data[2][1];
        (new_x, new_y)
    }

    /// 返回原始数据引用。
    pub fn as_array(&self) -> &[[f64; 3]; 3] {
        &self.data
    }
}

impl Default for Matrix3x3 {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn test_identity() {
        let m = Matrix3x3::identity();
        assert_eq!(m.get(0, 0), 1.0);
        assert_eq!(m.get(1, 1), 1.0);
        assert_eq!(m.get(2, 2), 1.0);
        assert_eq!(m.get(0, 1), 0.0);
    }

    #[test]
    fn test_new() {
        let m = Matrix3x3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(m.get(0, 0), 1.0);
        assert_eq!(m.get(0, 1), 2.0);
        assert_eq!(m.get(1, 0), 3.0);
        assert_eq!(m.get(1, 1), 4.0);
        assert_eq!(m.get(2, 0), 5.0);
        assert_eq!(m.get(2, 1), 6.0);
    }

    #[test]
    fn test_from_ctm() {
        let ctm = [1.0, 0.0, 0.0, 1.0, 10.0, 20.0];
        let m = Matrix3x3::from_ctm(&ctm);
        assert_eq!(m.get(2, 0), 10.0);
        assert_eq!(m.get(2, 1), 20.0);
    }

    #[test]
    fn test_multiply_identity() {
        let m = Matrix3x3::new(2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
        let result = m.multiply(&Matrix3x3::identity());
        assert_eq!(m, result);
    }

    #[test]
    fn test_multiply() {
        let a = Matrix3x3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let b = Matrix3x3::new(7.0, 8.0, 9.0, 10.0, 11.0, 12.0);
        let c = a.multiply(&b);
        // 手动验证：
        // c[0][0] = 1*7 + 2*9 + 0*11 = 7 + 18 = 25
        // c[0][1] = 1*8 + 2*10 + 0*12 = 8 + 20 = 28
        // c[1][0] = 3*7 + 4*9 + 0*11 = 21 + 36 = 57
        // c[2][0] = 5*7 + 6*9 + 1*11 = 35 + 54 + 11 = 100
        assert!(approx_eq(c.get(0, 0), 25.0));
        assert!(approx_eq(c.get(0, 1), 28.0));
        assert!(approx_eq(c.get(1, 0), 57.0));
        assert!(approx_eq(c.get(2, 0), 100.0));
    }

    #[test]
    fn test_scale() {
        let m = Matrix3x3::identity();
        let scaled = m.scale(2.0, 3.0);
        assert!(approx_eq(scaled.get(0, 0), 2.0));
        assert!(approx_eq(scaled.get(1, 1), 3.0));
    }

    #[test]
    fn test_translate() {
        let m = Matrix3x3::identity();
        let moved = m.translate(10.0, 20.0);
        assert!(approx_eq(moved.get(2, 0), 10.0));
        assert!(approx_eq(moved.get(2, 1), 20.0));
    }

    #[test]
    fn test_transform_point_identity() {
        let m = Matrix3x3::identity();
        let p = m.transform_point(5.0, 10.0);
        assert!(approx_eq(*p.first(), 5.0));
        assert!(approx_eq(*p.second(), 10.0));
    }

    #[test]
    fn test_transform_point_translate() {
        let m = Matrix3x3::new(1.0, 0.0, 0.0, 1.0, 100.0, 200.0);
        let p = m.transform_point(5.0, 10.0);
        assert!(approx_eq(*p.first(), 105.0));
        assert!(approx_eq(*p.second(), 210.0));
    }

    #[test]
    fn test_transform_point_scale() {
        let m = Matrix3x3::new(2.0, 0.0, 0.0, 3.0, 0.0, 0.0);
        let p = m.transform_point(5.0, 10.0);
        assert!(approx_eq(*p.first(), 10.0));
        assert!(approx_eq(*p.second(), 30.0));
    }

    #[test]
    fn test_mirror() {
        let m = Matrix3x3::identity();
        // 关于 Y 轴镜像 (a=1, b=0, c=0 → x=0 线)
        let mirrored = m.mirror(1.0, 0.0, 0.0);
        let p = mirrored.transform_point(5.0, 10.0);
        assert!(approx_eq(*p.first(), -5.0));
        assert!(approx_eq(*p.second(), 10.0));
    }

    #[test]
    fn test_left_top() {
        let m = Matrix3x3::identity();
        let lt = m.left_top();
        assert!(approx_eq(*lt.first(), 0.0));
        assert!(approx_eq(*lt.second(), 0.0));
    }

    #[test]
    fn test_set() {
        let mut m = Matrix3x3::identity();
        m.set(1, 2, 42.0);
        assert!(approx_eq(m.get(1, 2), 42.0));
    }

    #[test]
    fn test_default() {
        let m = Matrix3x3::default();
        assert_eq!(m, Matrix3x3::identity());
    }

    #[test]
    fn test_as_array() {
        let m = Matrix3x3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let arr = m.as_array();
        assert_eq!(arr[0][0], 1.0);
        assert_eq!(arr[2][0], 5.0);
    }
}
