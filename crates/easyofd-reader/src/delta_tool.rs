//! DeltaX 和 DeltaY 工具。
//!
//! 对应 Java: org.ofdrw.reader.DeltaTool
//!
//! 解析 OFD 文本对象中的 DeltaX/DeltaY 偏移数组，将压缩格式展开为
//! 逐字符的偏移值列表。

/// 获取 Delta 偏移数据。
///
/// 对应 Java: `DeltaTool.getDelta(ST_Array, int)`
///
/// OFD 的 DeltaX/DeltaY 数组支持压缩格式：当连续偏移值相同时，
/// 用 `"g"` 标记后跟重复次数和值（如 `["g", "5", "10.0"]` 表示
/// 5 次偏移值 10.0）。本函数将压缩格式展开为完整的偏移列表。
///
/// # 参数
///
/// - `delta`: 偏移数组的字符串表示（OFD ST_Array 格式）。
/// - `content_length`: 文本内容长度，用于补齐不足的偏移值。
#[must_use]
pub fn get_delta(delta: Option<&[String]>, content_length: usize) -> Vec<f64> {
    let mut list = Vec::new();

    if let Some(array) = delta {
        let mut i = 0;
        while i < array.len() {
            if array[i] == "g" && i + 2 < array.len() {
                // 压缩格式: "g" count value
                let count: usize = array[i + 1].parse().unwrap_or(0);
                let value: f64 = array[i + 2].parse().unwrap_or(0.0);
                for _ in 0..count {
                    list.push(value);
                }
                i += 3;
            } else {
                let value: f64 = array[i].parse().unwrap_or(0.0);
                list.push(value);
                i += 1;
            }
        }
    }

    // 如果偏移值不足，用最后一个值补齐
    let delta_size = list.len();
    if delta_size < content_length && delta_size > 0 {
        let last_delta = list[delta_size - 1];
        for _ in 0..(content_length - delta_size) {
            list.push(last_delta);
        }
    }

    list
}

/// 从 ST_Array 字符串格式解析偏移值（直接传入原始字符串）。
///
/// 辅助函数，将 OFD XML 中常见的逗号或空格分隔的数字字符串解析为 f64 列表。
#[must_use]
pub fn parse_delta_string(s: &str, content_length: usize) -> Vec<f64> {
    let parts: Vec<String> = s
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    get_delta(Some(&parts), content_length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_delta_none() {
        let result = get_delta(None, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_delta_simple() {
        let delta = vec!["10.0".into(), "12.0".into(), "11.5".into()];
        let result = get_delta(Some(&delta), 3);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 10.0).abs() < f64::EPSILON);
        assert!((result[1] - 12.0).abs() < f64::EPSILON);
        assert!((result[2] - 11.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_delta_compressed() {
        // "g" 3 10.0 表示 3 次偏移值 10.0
        let delta = vec!["g".into(), "3".into(), "10.0".into()];
        let result = get_delta(Some(&delta), 3);
        assert_eq!(result.len(), 3);
        for v in &result {
            assert!((v - 10.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_get_delta_pad_last() {
        let delta = vec!["5.0".into(), "6.0".into()];
        let result = get_delta(Some(&delta), 4);
        assert_eq!(result.len(), 4);
        assert!((result[0] - 5.0).abs() < f64::EPSILON);
        assert!((result[1] - 6.0).abs() < f64::EPSILON);
        assert!((result[2] - 6.0).abs() < f64::EPSILON);
        assert!((result[3] - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_delta_empty_array() {
        let delta: Vec<String> = vec![];
        let result = get_delta(Some(&delta), 3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_delta_string() {
        let result = parse_delta_string("1.0, 2.0, 3.0", 3);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_delta_string_space_separated() {
        let result = parse_delta_string("1.5 2.5 3.5", 3);
        assert_eq!(result.len(), 3);
        assert!((result[1] - 2.5).abs() < f64::EPSILON);
    }
}
