//! 基于 Fontations 的原生字体解析与度量。
use skrifa::{
    FontRef, MetadataProvider,
    instance::{LocationRef, Size},
    raw::{ReadError, TableProvider},
};

/// OpenType 字形编号，范围由字体的 maxp 表决定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphId(pub u16);

/// 借用 TTF、OTF 或 TTC 数据的真实 Fontations 字体。
pub(crate) struct FontFace<'a> {
    pub(crate) font: FontRef<'a>,
    charmap: skrifa::charmap::Charmap<'a>,
    metrics: skrifa::metrics::GlyphMetrics<'a>,
    units_per_em: u16,
    glyph_count: u16,
}
impl<'a> FontFace<'a> {
    /// 解析字体；参数为字体数据及集合索引，返回字体或表解析错误。
    pub(crate) fn parse(data: &'a [u8], index: u32) -> Result<Self, ReadError> {
        let font = FontRef::from_index(data, index)?;
        let head = font.head()?;
        if head.units_per_em() == 0 {
            return Err(ReadError::MalformedData("zero units per em"));
        }
        let glyph_count = font.maxp()?.num_glyphs();
        font.hhea()?;
        font.hmtx()?;
        font.cmap()?;
        let units_per_em = head.units_per_em();
        let charmap = font.charmap();
        let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
        Ok(Self {
            font,
            charmap,
            metrics,
            units_per_em,
            glyph_count,
        })
    }
    /// 返回每 em 的字体单位数，无参数。
    pub(crate) fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
    /// 返回 maxp 声明的字形数，无参数。
    pub(crate) fn number_of_glyphs(&self) -> u16 {
        self.glyph_count
    }
    /// 将参数 Unicode 字符映射为字形；未映射或越界返回 None。
    pub(crate) fn glyph_index(&self, character: char) -> Option<GlyphId> {
        let id = self.charmap.map(character)?.to_u32();
        (id > 0 && id < u32::from(self.number_of_glyphs())).then_some(GlyphId(id as u16))
    }
    /// 返回参数字形的水平推进量，单位为字体单位；越界返回 None。
    pub(crate) fn glyph_hor_advance(&self, glyph: GlyphId) -> Option<f32> {
        if glyph.0 >= self.number_of_glyphs() {
            return None;
        }
        self.metrics
            .advance_width(skrifa::GlyphId::new(u32::from(glyph.0)))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::FontFace;
    use std::io::Read;

    pub(crate) fn fixture_font() -> Vec<u8> {
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/upstream_20261014/stroke_true.ofd");
        let mut archive =
            zip::ZipArchive::new(std::io::Cursor::new(std::fs::read(source).unwrap())).unwrap();
        let mut data = Vec::new();
        archive
            .by_name("Doc_0/Res/font.ttf")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        data
    }

    /// 保留真实字形/度量，仅用确定性cmap将A轮廓映射到两枚CJK字符。
    pub(crate) fn cjk_fixture() -> Vec<u8> {
        let mut data = fixture_font();
        let gid = FontFace::parse(&data, 0)
            .unwrap()
            .glyph_index('A')
            .unwrap()
            .0;
        let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 1, 0, 0, 0, 12];
        let chars = ['中' as u16, '文' as u16];
        let mut values = vec![4_u16, 40, 0, 6, 4, 1, 2];
        values.extend([chars[0], chars[1], 0xffff, 0]);
        values.extend([chars[0], chars[1], 0xffff]);
        values.extend([gid.wrapping_sub(chars[0]), gid.wrapping_sub(chars[1]), 1]);
        values.extend([0, 0, 0]);
        for value in values {
            cmap.extend(value.to_be_bytes());
        }
        let tables = u16::from_be_bytes(data[4..6].try_into().unwrap());
        let entry = (0..usize::from(tables))
            .map(|i| 12 + i * 16)
            .find(|i| &data[*i..*i + 4] == b"cmap")
            .unwrap();
        let offset = data.len() as u32;
        data[entry + 8..entry + 12].copy_from_slice(&offset.to_be_bytes());
        data[entry + 12..entry + 16].copy_from_slice(&(cmap.len() as u32).to_be_bytes());
        data.extend(cmap);
        data
    }

    #[test]
    fn invalid_fonts_and_collection_indices_return_errors() {
        for data in [&[][..], &[0, 1, 0, 0][..], b"bad font".as_slice()] {
            assert!(FontFace::parse(data, 0).is_err());
        }
        let data = fixture_font();
        assert!(FontFace::parse(&data, 1).is_err());
    }

    #[test]
    fn ttc_selects_real_faces_and_rejects_out_of_range_index() {
        let first = fixture_font();
        let mut second = first.clone();
        let count = usize::from(u16::from_be_bytes(second[4..6].try_into().unwrap()));
        let head = (0..count)
            .map(|i| 12 + i * 16)
            .find(|i| &second[*i..*i + 4] == b"head")
            .unwrap();
        let offset = u32::from_be_bytes(second[head + 8..head + 12].try_into().unwrap()) as usize;
        second[offset + 18..offset + 20].copy_from_slice(&512_u16.to_be_bytes());
        let mut fonts = [first, second];
        let offsets = [20_u32, 20 + fonts[0].len() as u32];
        for (font, base) in fonts.iter_mut().zip(offsets) {
            for i in 0..count {
                let pos = 12 + i * 16 + 8;
                let offset = u32::from_be_bytes(font[pos..pos + 4].try_into().unwrap());
                font[pos..pos + 4].copy_from_slice(&(offset + base).to_be_bytes());
            }
        }
        let mut collection = b"ttcf".to_vec();
        for value in [0x10000_u32, 2, offsets[0], offsets[1]] {
            collection.extend(value.to_be_bytes());
        }
        for font in fonts {
            collection.extend(font);
        }
        for index in [0, 1] {
            let face = FontFace::parse(&collection, index).unwrap();
            assert_eq!(face.units_per_em(), if index == 0 { 256 } else { 512 });
            assert_eq!(face.number_of_glyphs(), 1384);
            let gid = face.glyph_index('A').unwrap();
            assert_eq!(gid.0, 102);
            assert_eq!(face.glyph_hor_advance(gid), Some(128.));
            assert!(
                super::super::glyph_outline::GlyphOutline::load(&face, gid)
                    .unwrap()
                    .path
                    .is_some()
            );
        }
        assert!(FontFace::parse(&collection, 2).is_err());
    }

    #[test]
    fn otf_named_and_cid_cff_have_real_outlines_and_metrics() {
        for data in [
            include_bytes!("../../tests/fixtures/fontations/mock_cff_named.otf").as_slice(),
            include_bytes!("../../tests/fixtures/fontations/mock_cff_cid.otf").as_slice(),
        ] {
            let face = FontFace::parse(data, 0).unwrap();
            assert_eq!(face.units_per_em(), 1000);
            assert_eq!(face.number_of_glyphs(), 12);
            let gid = face.glyph_index('A').unwrap();
            assert_eq!(gid.0, 2);
            assert_eq!(face.glyph_hor_advance(gid), Some(300.));
            let outline = super::super::glyph_outline::GlyphOutline::load(&face, gid).unwrap();
            assert!(!outline.svg.is_empty());
            let bounds = outline.path.unwrap().compute_tight_bounds().unwrap();
            assert_eq!(bounds.width().to_bits(), 300_f32.to_bits());
            let bitmap =
                super::super::glyph_bitmap::GlyphBitmap::rasterize(&face, 'A', 20.).unwrap();
            assert_eq!(bitmap.advance.to_bits(), 6_f32.to_bits());
            assert!(
                bitmap
                    .pixels
                    .unwrap()
                    .pixels()
                    .iter()
                    .any(|p| p.alpha() != 0)
            );
            let space =
                super::super::glyph_bitmap::GlyphBitmap::rasterize(&face, ' ', 20.).unwrap();
            assert_eq!(space.advance.to_bits(), 5_f32.to_bits());
            assert!(space.pixels.is_none());
        }
    }

    #[test]
    fn cjk_mapping_keeps_real_outline_and_advance() {
        let data = cjk_fixture();
        let face = FontFace::parse(&data, 0).unwrap();
        for character in ['中', '文'] {
            let gid = face.glyph_index(character).unwrap();
            assert_eq!(gid.0, 102);
            assert_eq!(face.glyph_hor_advance(gid), Some(128.));
            assert!(
                super::super::glyph_outline::GlyphOutline::load(&face, gid)
                    .unwrap()
                    .path
                    .is_some()
            );
        }
    }
}
