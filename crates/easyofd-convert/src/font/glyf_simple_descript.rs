//! 简单字形描述符。
//!
//! 对应 Java: org.ofdrw.converter.font.GlyfSimpleDescript

/// 简单字形描述符。
///
/// 对应 Java `GlyfSimpleDescript`。描述一个由单一轮廓集组成的字形，
/// 包含端点索引、坐标点和标志位。
///
/// 参考 OpenType `glyf` 表规范中简单字形的数据格式。
#[derive(Debug, Clone)]
pub struct GlyfSimpleDescript {
    /// 轮廓数量。
    contour_count: i16,
    /// 每个轮廓的端点索引。
    end_pts_of_contours: Vec<u16>,
    /// 坐标 X 值。
    x_coordinates: Vec<i16>,
    /// 坐标 Y 值。
    y_coordinates: Vec<i16>,
    /// 每个点的标志位。
    flags: Vec<u8>,
    /// 提示指令。
    instructions: Vec<u8>,
}

impl GlyfSimpleDescript {
    /// 创建简单字形描述符。
    ///
    /// # 参数
    /// - `contour_count`：轮廓数量（必须 >= 0）
    /// - `end_pts_of_contours`：每个轮廓的端点索引
    /// - `flags`：每个点的标志位
    /// - `x_coordinates`：X 坐标
    /// - `y_coordinates`：Y 坐标
    /// - `instructions`：提示指令
    pub fn new(
        contour_count: i16,
        end_pts_of_contours: Vec<u16>,
        flags: Vec<u8>,
        x_coordinates: Vec<i16>,
        y_coordinates: Vec<i16>,
        instructions: Vec<u8>,
    ) -> Self {
        Self {
            contour_count,
            end_pts_of_contours,
            x_coordinates,
            y_coordinates,
            flags,
            instructions,
        }
    }

    /// 创建空的简单字形描述符。
    pub fn empty() -> Self {
        Self {
            contour_count: 0,
            end_pts_of_contours: Vec::new(),
            x_coordinates: Vec::new(),
            y_coordinates: Vec::new(),
            flags: Vec::new(),
            instructions: Vec::new(),
        }
    }

    /// 返回轮廓数量。
    pub fn contour_count(&self) -> i16 {
        self.contour_count
    }

    /// 返回端点索引列表。
    pub fn end_pts_of_contours(&self) -> &[u16] {
        &self.end_pts_of_contours
    }

    /// 返回 X 坐标列表。
    pub fn x_coordinates(&self) -> &[i16] {
        &self.x_coordinates
    }

    /// 返回 Y 坐标列表。
    pub fn y_coordinates(&self) -> &[i16] {
        &self.y_coordinates
    }

    /// 返回标志位列表。
    pub fn flags(&self) -> &[u8] {
        &self.flags
    }

    /// 返回提示指令。
    pub fn instructions(&self) -> &[u8] {
        &self.instructions
    }

    /// 返回点的数量。
    pub fn point_count(&self) -> usize {
        self.flags.len()
    }

    /// 返回指定索引的点坐标。
    pub fn point(&self, index: usize) -> Option<(i16, i16)> {
        if index < self.x_coordinates.len() && index < self.y_coordinates.len() {
            Some((self.x_coordinates[index], self.y_coordinates[index]))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let desc = GlyfSimpleDescript::new(
            2,
            vec![5, 11],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![0, 100, 100, 0, 0, 50, 150, 150, 50, 50, 100, 100],
            vec![0, 0, 100, 100, 0, 50, 50, 150, 150, 50, 50, 100],
            vec![],
        );
        assert_eq!(desc.contour_count(), 2);
        assert_eq!(desc.point_count(), 12);
        assert_eq!(desc.end_pts_of_contours(), &[5, 11]);
    }

    #[test]
    fn test_empty() {
        let desc = GlyfSimpleDescript::empty();
        assert_eq!(desc.contour_count(), 0);
        assert_eq!(desc.point_count(), 0);
        assert!(desc.instructions().is_empty());
    }

    #[test]
    fn test_point_access() {
        let desc = GlyfSimpleDescript::new(
            1,
            vec![3],
            vec![1, 1, 1, 1],
            vec![10, 20, 30, 40],
            vec![50, 60, 70, 80],
            vec![],
        );
        assert_eq!(desc.point(0), Some((10, 50)));
        assert_eq!(desc.point(3), Some((40, 80)));
        assert_eq!(desc.point(4), None);
    }

    #[test]
    fn test_clone() {
        let desc = GlyfSimpleDescript::new(
            1,
            vec![2],
            vec![1, 1, 1],
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![],
        );
        let desc2 = desc.clone();
        assert_eq!(desc.contour_count(), desc2.contour_count());
        assert_eq!(desc.point_count(), desc2.point_count());
    }
}
