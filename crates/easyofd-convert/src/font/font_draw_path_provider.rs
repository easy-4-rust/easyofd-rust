//! 字形绘制路径提供者。
//!
//! 对应 Java: org.ofdrw.converter.font.FontDrawPathProvider
//!
//! Java 版是一个接口，提供从字体中获取字形轮廓路径的能力。
//! Rust 版定义为 trait，允许不同的字体后端提供字形路径数据。

/// 字形轮廓点。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphPoint {
    /// X 坐标。
    pub x: f64,
    /// Y 坐标。
    pub y: f64,
    /// 是否为控制点（贝塞尔曲线控制点）。
    pub on_curve: bool,
}

/// 字形轮廓路径。
#[derive(Debug, Clone, Default)]
pub struct GlyphPath {
    /// 轮廓点列表。
    pub points: Vec<GlyphPoint>,
    /// 是否闭合路径。
    pub closed: bool,
}

/// 字形绘制路径提供者 trait。
///
/// 对应 Java: `org.ofdrw.converter.font.FontDrawPathProvider`
///
/// 实现此 trait 的类型能够从字体中提取指定字形的轮廓路径。
/// 路径用于 SVG 导出、PDF 文本渲染（当需要轮廓而非嵌入字体时）等场景。
pub trait FontDrawPathProvider {
    /// 获取指定 Unicode 码点的字形轮廓路径。
    ///
    /// # 参数
    /// - `code_point`: Unicode 码点
    ///
    /// # 返回
    /// 字形轮廓路径。如果字形不存在或无轮廓数据则返回 `None`。
    fn glyph_path(&self, code_point: u32) -> Option<GlyphPath>;

    /// 获取指定字形 ID 的轮廓路径。
    ///
    /// # 参数
    /// - `glyph_id`: 字体内部字形 ID
    ///
    /// # 返回
    /// 字形轮廓路径。
    fn glyph_path_by_id(&self, glyph_id: u32) -> Option<GlyphPath>;

    /// 字体是否包含指定 Unicode 码点的字形。
    fn has_glyph(&self, code_point: u32) -> bool {
        self.glyph_path(code_point).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPathProvider;

    impl FontDrawPathProvider for MockPathProvider {
        fn glyph_path(&self, code_point: u32) -> Option<GlyphPath> {
            if code_point == 0x41 {
                // 'A' 的简化轮廓
                Some(GlyphPath {
                    points: vec![
                        GlyphPoint {
                            x: 0.0,
                            y: 0.0,
                            on_curve: true,
                        },
                        GlyphPoint {
                            x: 500.0,
                            y: 700.0,
                            on_curve: true,
                        },
                        GlyphPoint {
                            x: 1000.0,
                            y: 0.0,
                            on_curve: true,
                        },
                    ],
                    closed: true,
                })
            } else {
                None
            }
        }

        fn glyph_path_by_id(&self, _glyph_id: u32) -> Option<GlyphPath> {
            None
        }
    }

    #[test]
    fn test_has_glyph() {
        let provider = MockPathProvider;
        assert!(provider.has_glyph(0x41)); // 'A'
        assert!(!provider.has_glyph(0x42)); // 'B'
    }

    #[test]
    fn test_glyph_path() {
        let provider = MockPathProvider;
        let path = provider.glyph_path(0x41).unwrap();
        assert_eq!(path.points.len(), 3);
        assert!(path.closed);
        assert!(path.points[0].on_curve);
    }

    #[test]
    fn test_glyph_path_none() {
        let provider = MockPathProvider;
        assert!(provider.glyph_path(0x9999).is_none());
    }

    #[test]
    fn test_glyph_path_by_id() {
        let provider = MockPathProvider;
        assert!(provider.glyph_path_by_id(0).is_none());
    }

    #[test]
    fn test_glyph_point_copy() {
        let p = GlyphPoint {
            x: 1.0,
            y: 2.0,
            on_curve: true,
        };
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn test_glyph_path_default() {
        let path = GlyphPath::default();
        assert!(path.points.is_empty());
        assert!(!path.closed);
    }
}
