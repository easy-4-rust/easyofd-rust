//! 命名颜色解析工具。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.NamedColor
//!
//! 支持 CSS 命名颜色、`#RGB`、`#RRGGBB`、`rgb(r,g,b)`、`rgba(r,g,b,a)` 格式解析。

/// 命名颜色解析工具。
///
/// 对应 Java: ofdrw layout canvas NamedColor。
pub struct NamedColor;

impl NamedColor {
    /// 解析颜色字符串为 RGB 或 RGBA 分量。
    ///
    /// 返回 `[r, g, b]` 或 `[r, g, b, a]`（a 为 0-255 的透明度）。
    /// 若颜色无法解析则返回 `None`。
    ///
    /// 支持格式：
    /// - CSS 命名颜色（如 `"red"`、`"aliceblue"`）
    /// - `#RGB`（如 `"#F00"`）
    /// - `#RRGGBB`（如 `"#FF0000"`）
    /// - `rgb(r,g,b)`（如 `"rgb(255,0,0)"`）
    /// - `rgba(r,g,b,a)`（如 `"rgba(255,0,0,0.5)"`）
    #[must_use]
    // rgba 的 alpha 已 clamp 到 [0, 255]，cast 安全。
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn rgb(hex: &str) -> Option<Vec<u8>> {
        if hex.is_empty() {
            return None;
        }
        let hex = hex.to_lowercase();
        let hex = hex.trim();

        // 尝试命名颜色
        if let Some(rgb) = Self::named_color(hex) {
            return Some(rgb);
        }

        // rgb(r,g,b) 格式
        if let Some(s) = hex.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() == 3 {
                let r = parts[0].trim().parse::<u8>().ok()?;
                let g = parts[1].trim().parse::<u8>().ok()?;
                let b = parts[2].trim().parse::<u8>().ok()?;
                return Some(vec![r, g, b]);
            }
        }

        // rgba(r,g,b,a) 格式
        if let Some(s) = hex.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() == 4 {
                let r = parts[0].trim().parse::<u8>().ok()?;
                let g = parts[1].trim().parse::<u8>().ok()?;
                let b = parts[2].trim().parse::<u8>().ok()?;
                let a_f = parts[3].trim().parse::<f64>().ok()?;
                let a = (a_f * 255.0).round().clamp(0.0, 255.0) as u32 as u8;
                return Some(vec![r, g, b, a]);
            }
        }

        // #RGB / #RRGGBB 格式
        let short = hex.strip_prefix('#')?;
        if short.len() == 3 {
            let r = u8::from_str_radix(&short[0..1], 16).ok()?;
            let g = u8::from_str_radix(&short[1..2], 16).ok()?;
            let b = u8::from_str_radix(&short[2..3], 16).ok()?;
            return Some(vec![
                r.wrapping_mul(17),
                g.wrapping_mul(17),
                b.wrapping_mul(17),
            ]);
        }
        if short.len() == 6 {
            let r = u8::from_str_radix(&short[0..2], 16).ok()?;
            let g = u8::from_str_radix(&short[2..4], 16).ok()?;
            let b = u8::from_str_radix(&short[4..6], 16).ok()?;
            return Some(vec![r, g, b]);
        }
        None
    }

    /// 解析 CSS 命名颜色。
    fn named_color(name: &str) -> Option<Vec<u8>> {
        match name {
            "aliceblue" => Some(vec![240, 248, 255]),
            "antiquewhite" => Some(vec![250, 235, 215]),
            "aqua" | "cyan" => Some(vec![0, 255, 255]),
            "aquamarine" => Some(vec![127, 255, 212]),
            "azure" => Some(vec![240, 255, 255]),
            "beige" => Some(vec![245, 245, 220]),
            "bisque" => Some(vec![255, 228, 196]),
            "black" => Some(vec![0, 0, 0]),
            "blanchedalmond" => Some(vec![255, 235, 205]),
            "blue" => Some(vec![0, 0, 255]),
            "blueviolet" => Some(vec![138, 43, 226]),
            "brown" => Some(vec![165, 42, 42]),
            "burlywood" => Some(vec![222, 184, 135]),
            "cadetblue" => Some(vec![95, 158, 160]),
            "chartreuse" => Some(vec![127, 255, 0]),
            "chocolate" => Some(vec![210, 105, 30]),
            "coral" => Some(vec![255, 127, 80]),
            "cornflowerblue" => Some(vec![100, 149, 237]),
            "cornsilk" => Some(vec![255, 248, 220]),
            "crimson" => Some(vec![220, 20, 60]),
            "darkblue" => Some(vec![0, 0, 139]),
            "darkcyan" => Some(vec![0, 139, 139]),
            "darkgoldenrod" => Some(vec![184, 134, 11]),
            "darkgray" | "darkgrey" => Some(vec![169, 169, 169]),
            "darkgreen" => Some(vec![0, 100, 0]),
            "darkkhaki" => Some(vec![189, 183, 107]),
            "darkmagenta" => Some(vec![139, 0, 139]),
            "darkolivegreen" => Some(vec![85, 107, 47]),
            "darkorange" => Some(vec![255, 140, 0]),
            "darkorchid" => Some(vec![153, 50, 204]),
            "darkred" => Some(vec![139, 0, 0]),
            "darksalmon" => Some(vec![233, 150, 122]),
            "darkseagreen" => Some(vec![143, 188, 143]),
            "darkslateblue" => Some(vec![72, 61, 139]),
            "darkslategray" | "darkslategrey" => Some(vec![47, 79, 79]),
            "darkturquoise" => Some(vec![0, 206, 209]),
            "darkviolet" => Some(vec![148, 0, 211]),
            "deeppink" => Some(vec![255, 20, 147]),
            "deepskyblue" => Some(vec![0, 191, 255]),
            "dimgray" | "dimgrey" => Some(vec![105, 105, 105]),
            "dodgerblue" => Some(vec![30, 144, 255]),
            "firebrick" => Some(vec![178, 34, 34]),
            "floralwhite" => Some(vec![255, 250, 240]),
            "forestgreen" => Some(vec![34, 139, 34]),
            "fuchsia" | "magenta" => Some(vec![255, 0, 255]),
            "gainsboro" => Some(vec![220, 220, 220]),
            "ghostwhite" => Some(vec![248, 248, 255]),
            "gold" => Some(vec![255, 215, 0]),
            "goldenrod" => Some(vec![218, 165, 32]),
            "gray" | "grey" => Some(vec![128, 128, 128]),
            "green" => Some(vec![0, 128, 0]),
            "greenyellow" => Some(vec![173, 255, 47]),
            "honeydew" => Some(vec![240, 255, 240]),
            "hotpink" => Some(vec![255, 105, 180]),
            "indianred" => Some(vec![205, 92, 92]),
            "indigo" => Some(vec![75, 0, 130]),
            "ivory" => Some(vec![255, 255, 240]),
            "khaki" => Some(vec![240, 230, 140]),
            "lavender" => Some(vec![230, 230, 250]),
            "lavenderblush" => Some(vec![255, 240, 245]),
            "lawngreen" => Some(vec![124, 252, 0]),
            "lemonchiffon" => Some(vec![255, 250, 205]),
            "lightblue" => Some(vec![173, 216, 230]),
            "lightcoral" => Some(vec![240, 128, 128]),
            "lightcyan" => Some(vec![224, 255, 255]),
            "lightgoldenrodyellow" => Some(vec![250, 250, 210]),
            "lightgray" | "lightgrey" => Some(vec![211, 211, 211]),
            "lightgreen" => Some(vec![144, 238, 144]),
            "lightpink" => Some(vec![255, 182, 193]),
            "lightsalmon" => Some(vec![255, 160, 122]),
            "lightseagreen" => Some(vec![32, 178, 170]),
            "lightskyblue" => Some(vec![135, 206, 250]),
            "lightslategray" | "lightslategrey" => Some(vec![119, 136, 153]),
            "lightsteelblue" => Some(vec![176, 196, 222]),
            "lightyellow" => Some(vec![255, 255, 224]),
            "lime" => Some(vec![0, 255, 0]),
            "limegreen" => Some(vec![50, 205, 50]),
            "linen" => Some(vec![250, 240, 230]),
            "maroon" => Some(vec![128, 0, 0]),
            "mediumaquamarine" => Some(vec![102, 205, 170]),
            "mediumblue" => Some(vec![0, 0, 205]),
            "mediumorchid" => Some(vec![186, 85, 211]),
            "mediumpurple" => Some(vec![147, 112, 216]),
            "mediumseagreen" => Some(vec![60, 179, 113]),
            "mediumslateblue" => Some(vec![123, 104, 238]),
            "mediumspringgreen" => Some(vec![0, 250, 154]),
            "mediumturquoise" => Some(vec![72, 209, 204]),
            "mediumvioletred" => Some(vec![199, 21, 133]),
            "midnightblue" => Some(vec![25, 25, 112]),
            "mintcream" => Some(vec![245, 255, 250]),
            "mistyrose" => Some(vec![255, 228, 225]),
            "moccasin" => Some(vec![255, 228, 181]),
            "navajowhite" => Some(vec![255, 222, 173]),
            "navy" => Some(vec![0, 0, 128]),
            "oldlace" => Some(vec![253, 245, 230]),
            "olive" => Some(vec![128, 128, 0]),
            "olivedrab" => Some(vec![107, 142, 35]),
            "orange" => Some(vec![255, 165, 0]),
            "orangered" => Some(vec![255, 69, 0]),
            "orchid" => Some(vec![218, 112, 214]),
            "palegoldenrod" => Some(vec![238, 232, 170]),
            "palegreen" => Some(vec![152, 251, 152]),
            "paleturquoise" => Some(vec![175, 238, 238]),
            "palevioletred" => Some(vec![216, 112, 147]),
            "papayawhip" => Some(vec![255, 239, 213]),
            "peachpuff" => Some(vec![255, 218, 185]),
            "peru" => Some(vec![205, 133, 63]),
            "pink" => Some(vec![255, 192, 203]),
            "plum" => Some(vec![221, 160, 221]),
            "powderblue" => Some(vec![176, 224, 230]),
            "purple" => Some(vec![128, 0, 128]),
            "red" => Some(vec![255, 0, 0]),
            "rosybrown" => Some(vec![188, 143, 143]),
            "royalblue" => Some(vec![65, 105, 225]),
            "saddlebrown" => Some(vec![139, 69, 19]),
            "salmon" => Some(vec![250, 128, 114]),
            "sandybrown" => Some(vec![244, 164, 96]),
            "seagreen" => Some(vec![46, 139, 87]),
            "seashell" => Some(vec![255, 245, 238]),
            "sienna" => Some(vec![160, 82, 45]),
            "silver" => Some(vec![192, 192, 192]),
            "skyblue" => Some(vec![135, 206, 235]),
            "slateblue" => Some(vec![106, 90, 205]),
            "slategray" | "slategrey" => Some(vec![112, 128, 144]),
            "snow" => Some(vec![255, 250, 250]),
            "springgreen" => Some(vec![0, 255, 127]),
            "steelblue" => Some(vec![70, 130, 180]),
            "tan" => Some(vec![210, 180, 140]),
            "teal" => Some(vec![0, 128, 128]),
            "thistle" => Some(vec![216, 191, 216]),
            "tomato" => Some(vec![255, 99, 71]),
            "turquoise" => Some(vec![64, 224, 208]),
            "violet" => Some(vec![238, 130, 238]),
            "wheat" => Some(vec![245, 222, 179]),
            "white" => Some(vec![255, 255, 255]),
            "whitesmoke" => Some(vec![245, 245, 245]),
            "yellow" => Some(vec![255, 255, 0]),
            "yellowgreen" => Some(vec![154, 205, 50]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_color_black() {
        let rgb = NamedColor::rgb("black").unwrap();
        assert_eq!(rgb, vec![0, 0, 0]);
    }

    #[test]
    fn test_named_color_white() {
        let rgb = NamedColor::rgb("white").unwrap();
        assert_eq!(rgb, vec![255, 255, 255]);
    }

    #[test]
    fn test_named_color_red() {
        let rgb = NamedColor::rgb("red").unwrap();
        assert_eq!(rgb, vec![255, 0, 0]);
    }

    #[test]
    fn test_hex_rrggbb() {
        let rgb = NamedColor::rgb("#FF0000").unwrap();
        assert_eq!(rgb, vec![255, 0, 0]);
    }

    #[test]
    fn test_hex_rgb_short() {
        let rgb = NamedColor::rgb("#F00").unwrap();
        assert_eq!(rgb, vec![255, 0, 0]);
    }

    #[test]
    fn test_hex_lowercase() {
        let rgb = NamedColor::rgb("#ff8000").unwrap();
        assert_eq!(rgb, vec![255, 128, 0]);
    }

    #[test]
    fn test_rgb_format() {
        let rgb = NamedColor::rgb("rgb(10, 20, 30)").unwrap();
        assert_eq!(rgb, vec![10, 20, 30]);
    }

    #[test]
    fn test_rgba_format() {
        let rgb = NamedColor::rgb("rgba(255, 0, 0, 0.5)").unwrap();
        assert_eq!(rgb, vec![255, 0, 0, 128]);
    }

    #[test]
    fn test_empty_returns_none() {
        assert!(NamedColor::rgb("").is_none());
    }

    #[test]
    fn test_invalid_returns_none() {
        assert!(NamedColor::rgb("notacolor").is_none());
        assert!(NamedColor::rgb("#GGGGGG").is_none());
    }

    #[test]
    fn test_case_insensitive() {
        let rgb = NamedColor::rgb("RED").unwrap();
        assert_eq!(rgb, vec![255, 0, 0]);
        let rgb = NamedColor::rgb("Red").unwrap();
        assert_eq!(rgb, vec![255, 0, 0]);
    }
}
