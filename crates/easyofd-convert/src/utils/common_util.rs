//! 通用转换工具函数。
//!
//! 对应 Java: org.ofdrw.converter.utils.CommonUtil

/// 毫米转换为像素。
///
/// 对应 Java `CommonUtil.millimetersToPixel(mm, dpi)`。
///
/// # 参数
/// - `mm`：毫米值
/// - `dpi`：每英寸点数（如 72、96、300）
///
/// # 返回
/// 对应的像素值。
pub fn millimeters_to_pixel(mm: f64, dpi: f64) -> f64 {
    mm * dpi / 25.4
}

/// 像素转换为毫米。
///
/// 对应 Java `CommonUtil.pixelToMillimeters(px, dpi)`。
///
/// # 参数
/// - `px`：像素值
/// - `dpi`：每英寸点数
///
/// # 返回
/// 对应的毫米值。
pub fn pixel_to_millimeters(px: f64, dpi: f64) -> f64 {
    px * 25.4 / dpi
}

/// 获取指定 DPI 下的每毫米像素数量。
///
/// 对应 Java `CommonUtil.dpiToPpm(dpi)`。
///
/// # 参数
/// - `dpi`：每英寸像素（如 200、300）
///
/// # 返回
/// 像素每毫米。
pub fn dpi_to_ppm(dpi: u32) -> f64 {
    f64::from(dpi) * (0.01 / 0.254)
}

/// 将 `f64` 数组转换为 `f32` 数组。
///
/// 对应 Java `CommonUtil.doubleArrayToFloatArray(doubleArray)`。
pub fn f64_slice_to_f32(input: &[f64]) -> Vec<f32> {
    input.iter().map(|&v| v as f32).collect()
}

/// 将毫米坐标转换为 72 DPI 下的点坐标。
///
/// 对应 Java `CommonUtil.converterDpi(len)`。
pub fn converter_dpi(len: f64) -> f64 {
    millimeters_to_pixel(len, 72.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn test_millimeters_to_pixel_72dpi() {
        // 25.4mm = 1 inch = 72px at 72dpi
        assert!(approx_eq(millimeters_to_pixel(25.4, 72.0), 72.0));
    }

    #[test]
    fn test_millimeters_to_pixel_300dpi() {
        // 25.4mm at 300dpi = 300px
        assert!(approx_eq(millimeters_to_pixel(25.4, 300.0), 300.0));
    }

    #[test]
    fn test_pixel_to_millimeters() {
        assert!(approx_eq(pixel_to_millimeters(72.0, 72.0), 25.4));
        assert!(approx_eq(pixel_to_millimeters(300.0, 300.0), 25.4));
    }

    #[test]
    fn test_roundtrip_mm_px() {
        let mm = 100.0;
        let dpi = 96.0;
        let px = millimeters_to_pixel(mm, dpi);
        let back = pixel_to_millimeters(px, dpi);
        assert!(approx_eq(back, mm));
    }

    #[test]
    fn test_dpi_to_ppm() {
        // 72 dpi: 72 * 0.01/0.254 ≈ 2.8346
        let ppm = dpi_to_ppm(72);
        assert!(approx_eq(ppm, 2.834645669));
    }

    #[test]
    fn test_f64_slice_to_f32() {
        let input = [1.0_f64, 2.5, 3.7];
        let result = f64_slice_to_f32(&input);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 1.0_f32).abs() < f32::EPSILON);
        assert!((result[1] - 2.5_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_f64_slice_to_f32_empty() {
        let result = f64_slice_to_f32(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_converter_dpi() {
        // converterDpi(25.4) = 25.4 * 72 / 25.4 = 72.0
        assert!(approx_eq(converter_dpi(25.4), 72.0));
    }
}
