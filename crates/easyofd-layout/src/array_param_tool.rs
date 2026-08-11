//! 数组参数工具。
//!
//! 对应 Java: org.ofdrw.layout.element.ArrayParamTool

/// 数组参数工具，用于将不同长度的参数规范化为四元组。
///
/// 对应 Java: ofdrw layout ArrayParamTool。
pub struct ArrayParamTool;

impl ArrayParamTool {
    /// 四个参数分析工具（对应 Java: ArrayParamTool#arr4p）。
    ///
    /// 规范化规则：
    /// - 空时返回 `[0, 0, 0, 0]`
    /// - 1 个元素返回 `[a, a, a, a]`
    /// - 2 个元素返回 `[a, b, a, b]`
    /// - 3 个元素返回 `[a, b, c, 0]`
    /// - 4 个及以上返回前 4 个
    #[must_use]
    pub fn arr4p(arr: &[f64]) -> [f64; 4] {
        match arr.len() {
            0 => [0.0, 0.0, 0.0, 0.0],
            1 => [arr[0], arr[0], arr[0], arr[0]],
            2 => [arr[0], arr[1], arr[0], arr[1]],
            3 => [arr[0], arr[1], arr[2], 0.0],
            _ => [arr[0], arr[1], arr[2], arr[3]],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: &[f64; 4], b: &[f64; 4]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < f64::EPSILON)
    }

    #[test]
    fn test_empty() {
        assert!(approx(&ArrayParamTool::arr4p(&[]), &[0.0, 0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_one_element() {
        assert!(approx(
            &ArrayParamTool::arr4p(&[5.0]),
            &[5.0, 5.0, 5.0, 5.0]
        ));
    }

    #[test]
    fn test_two_elements() {
        assert!(approx(
            &ArrayParamTool::arr4p(&[1.0, 2.0]),
            &[1.0, 2.0, 1.0, 2.0]
        ));
    }

    #[test]
    fn test_three_elements() {
        assert!(approx(
            &ArrayParamTool::arr4p(&[1.0, 2.0, 3.0]),
            &[1.0, 2.0, 3.0, 0.0]
        ));
    }

    #[test]
    fn test_four_elements() {
        assert!(approx(
            &ArrayParamTool::arr4p(&[1.0, 2.0, 3.0, 4.0]),
            &[1.0, 2.0, 3.0, 4.0]
        ));
    }

    #[test]
    fn test_more_than_four() {
        assert!(approx(
            &ArrayParamTool::arr4p(&[1.0, 2.0, 3.0, 4.0, 5.0]),
            &[1.0, 2.0, 3.0, 4.0]
        ));
    }
}
