//! OFD 标准字体名称。
//!
//! 对应 Java: org.ofdrw.font.FontName

/// OFD 标准字体名称（ofdrw FontName 枚举）。
///
/// 对应 Java: ofdrw FontName。用于在版面引擎中指定标准字体，
/// 每个变体对应一个中文字体族名。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontName {
    /// 宋体（SimSun）。
    SimSun,
    /// 黑体（SimHei）。
    SimHei,
    /// 微软雅黑（MSYahei）。
    MSYahei,
    /// 楷体（KaiTi）。
    KaiTi,
    /// 仿宋（FangSong）。
    FangSong,
    /// Times New Roman（仅支持英文）。
    TimesNewRoman,
}

impl FontName {
    /// 返回字体的中文族名（对应 ofdrw `FontName.font()` 的 name）。
    ///
    /// 对应 Java: ofdrw FontName#font 返回的字体名。
    #[must_use]
    pub fn family_name(self) -> &'static str {
        match self {
            Self::SimSun => "宋体",
            Self::SimHei => "黑体",
            Self::MSYahei => "微软雅黑",
            Self::KaiTi => "楷体",
            Self::FangSong => "仿宋",
            Self::TimesNewRoman => "Times New Roman",
        }
    }

    /// 从字体族名解析 `FontName`（如 "宋体" → SimSun）。
    #[must_use]
    pub fn from_family_name(name: &str) -> Option<Self> {
        match name {
            "宋体" | "SimSun" => Some(Self::SimSun),
            "黑体" | "SimHei" => Some(Self::SimHei),
            "微软雅黑" | "MSYahei" => Some(Self::MSYahei),
            "楷体" | "KaiTi" => Some(Self::KaiTi),
            "仿宋" | "FangSong" => Some(Self::FangSong),
            "Times New Roman" => Some(Self::TimesNewRoman),
            _ => None,
        }
    }

    /// 可打印 ASCII 字符（区间 [32, 126]）的宽度占字体大小的比例。
    ///
    /// 对应 Java: ofdrw FontName#NOTO_PRINTABLE_ASCII_WIDTH_MAP。
    /// 空格为半个字符宽度（0.5）。
    #[must_use]
    pub fn printable_ascii_width(self) -> &'static [f64; 95] {
        match self {
            Self::TimesNewRoman => &TIMES_NEW_ROMAN_PRINTABLE_ASCII_MAP,
            _ => &NOTO_PRINTABLE_ASCII_WIDTH_MAP,
        }
    }
}

/// NOTO 字体可打印 ASCII 宽度表（字符 32..=126，索引 = char - 32）。
///
/// 对应 Java: ofdrw FontName#NOTO_PRINTABLE_ASCII_WIDTH_MAP。
/// 数值保持与 ofdrw 逐位一致，故不加字面量分隔符。
#[allow(clippy::unreadable_literal)]
pub static NOTO_PRINTABLE_ASCII_WIDTH_MAP: [f64; 95] = [
    0.5,
    0.3125,
    0.435546875,
    0.63818359375,
    0.58642578125,
    0.8896484375,
    0.8701171875,
    0.25634765625,
    0.333984375,
    0.333984375,
    0.455078125,
    0.74169921875,
    0.24072265625,
    0.4326171875,
    0.24072265625,
    0.42724609375,
    0.58642578125,
    0.58642578125,
    0.58642578125,
    0.58642578125,
    0.58642578125,
    0.58642578125,
    0.58642578125,
    0.58642578125,
    0.58642578125,
    0.58642578125,
    0.24072265625,
    0.24072265625,
    0.74169921875,
    0.74169921875,
    0.74169921875,
    0.48291015625,
    1.03125,
    0.70361328125,
    0.62744140625,
    0.6689453125,
    0.76171875,
    0.5498046875,
    0.53125,
    0.74365234375,
    0.7734375,
    0.2939453125,
    0.39599609375,
    0.634765625,
    0.51318359375,
    0.97705078125,
    0.81298828125,
    0.81494140625,
    0.61181640625,
    0.81494140625,
    0.65283203125,
    0.5771484375,
    0.5732421875,
    0.74658203125,
    0.67626953125,
    1.017578125,
    0.64501953125,
    0.603515625,
    0.6201171875,
    0.333984375,
    0.416015625,
    0.333984375,
    0.74169921875,
    0.4482421875,
    0.294921875,
    0.552734375,
    0.638671875,
    0.50146484375,
    0.6396484375,
    0.5673828125,
    0.3466796875,
    0.6396484375,
    0.61572265625,
    0.26611328125,
    0.26708984375,
    0.54443359375,
    0.26611328125,
    0.93701171875,
    0.6162109375,
    0.6357421875,
    0.638671875,
    0.6396484375,
    0.3818359375,
    0.462890625,
    0.37255859375,
    0.6162109375,
    0.52490234375,
    0.78955078125,
    0.5068359375,
    0.529296875,
    0.49169921875,
    0.333984375,
    0.26904296875,
    0.333984375,
    0.74169921875,
];

/// Times New Roman 字体可打印 ASCII 宽度表（字符 32..=126）。
///
/// 对应 Java: ofdrw FontName#TIMES_NEW_ROMAN_PRINTABLE_ASCII_MAP。
/// 数值保持与 ofdrw 逐位一致，故不加字面量分隔符。
#[allow(clippy::unreadable_literal)]
pub static TIMES_NEW_ROMAN_PRINTABLE_ASCII_MAP: [f64; 95] = [
    0.25,
    0.3330078125,
    0.408203125,
    0.5,
    0.5,
    0.8330078125,
    0.77783203125,
    0.18017578125,
    0.3330078125,
    0.3330078125,
    0.5,
    0.56396484375,
    0.25,
    0.3330078125,
    0.25,
    0.27783203125,
    0.5,
    0.46326171875,
    0.5,
    0.5,
    0.5,
    0.5,
    0.5,
    0.5,
    0.5,
    0.5,
    0.27783203125,
    0.27783203125,
    0.56396484375,
    0.56396484375,
    0.56396484375,
    0.44384765625,
    0.9208984375,
    0.72216796875,
    0.6669921875,
    0.6669921875,
    0.72216796875,
    0.61083984375,
    0.55615234375,
    0.72216796875,
    0.72216796875,
    0.3330078125,
    0.38916015625,
    0.72216796875,
    0.61083984375,
    0.88916015625,
    0.72216796875,
    0.72216796875,
    0.55615234375,
    0.72216796875,
    0.6669921875,
    0.55615234375,
    0.61083984375,
    0.72216796875,
    0.72216796875,
    0.94384765625,
    0.72216796875,
    0.72216796875,
    0.61083984375,
    0.3330078125,
    0.27783203125,
    0.3330078125,
    0.46923828125,
    0.5,
    0.3330078125,
    0.44384765625,
    0.5,
    0.44384765625,
    0.5,
    0.44384765625,
    0.3151220703125,
    0.5,
    0.5,
    0.27783203125,
    0.27783203125,
    0.5,
    0.27783203125,
    0.77783203125,
    0.5,
    0.5,
    0.5,
    0.5,
    0.3330078125,
    0.38916015625,
    0.27783203125,
    0.5,
    0.5,
    0.72216796875,
    0.5,
    0.5,
    0.44384765625,
    0.47998046875,
    0.2001953125,
    0.47998046875,
    0.541015625,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_name_mapping() {
        assert_eq!(FontName::SimSun.family_name(), "宋体");
        assert_eq!(FontName::SimHei.family_name(), "黑体");
        assert_eq!(FontName::KaiTi.family_name(), "楷体");
        assert_eq!(FontName::FangSong.family_name(), "仿宋");
        assert_eq!(FontName::TimesNewRoman.family_name(), "Times New Roman");
    }

    #[test]
    fn test_from_family_name() {
        assert_eq!(FontName::from_family_name("宋体"), Some(FontName::SimSun));
        assert_eq!(FontName::from_family_name("SimSun"), Some(FontName::SimSun));
        assert_eq!(FontName::from_family_name("楷体"), Some(FontName::KaiTi));
        assert_eq!(FontName::from_family_name("unknown"), None);
    }

    #[test]
    fn test_width_table_lengths() {
        assert_eq!(NOTO_PRINTABLE_ASCII_WIDTH_MAP.len(), 95);
        assert_eq!(TIMES_NEW_ROMAN_PRINTABLE_ASCII_MAP.len(), 95);
        // ASCII 区间 [32, 126]：空格为半字符宽，末字符宽度来自 ofdrw 表。
        assert!((FontName::SimSun.printable_ascii_width()[0] - 0.5).abs() < f64::EPSILON);
        assert!(
            (FontName::SimSun.printable_ascii_width()[94] - 0.741_699_218_75).abs() < f64::EPSILON
        );
    }
}
