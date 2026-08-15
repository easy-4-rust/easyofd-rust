//! cmap 编码子表。
//!
//! 对应 Java: org.ofdrw.converter.font.CmapSubtable
//!
//! 参考 OpenType `cmap` 表规范。支持 Format 0（字节编码）和 Format 4（段映射）两种常用格式。

/// cmap 子表。
///
/// 对应 Java `CmapSubtable`。将字符编码映射到字形索引。
#[derive(Debug, Clone)]
pub struct CmapSubtable {
    /// 平台 ID。
    platform_id: u16,
    /// 平台编码 ID。
    encoding_id: u16,
    /// 格式号。
    format: u16,
    /// Format 0：256 字节的字节编码映射。
    glyph_index_array: Vec<u8>,
    /// Format 4：起始码数组。
    start_code: Vec<u16>,
    /// Format 4：结束码数组。
    end_code: Vec<u16>,
    /// Format 4：ID 偏移数组。
    id_delta: Vec<i16>,
    /// Format 4：ID 范围偏移数组。
    id_range_offset: Vec<u16>,
    /// Format 4：字形索引数组。
    glyph_id_array: Vec<u16>,
}

impl CmapSubtable {
    /// 创建空的 cmap 子表。
    pub fn new(platform_id: u16, encoding_id: u16, format: u16) -> Self {
        Self {
            platform_id,
            encoding_id,
            format,
            glyph_index_array: Vec::new(),
            start_code: Vec::new(),
            end_code: Vec::new(),
            id_delta: Vec::new(),
            id_range_offset: Vec::new(),
            glyph_id_array: Vec::new(),
        }
    }

    // ─── Format 0 ────────────────────────────────────────────────────────────

    /// 设置 Format 0 字节编码映射。
    pub fn set_format0(&mut self, array: Vec<u8>) {
        self.glyph_index_array = array;
    }

    /// Format 0：通过字符编码获取字形索引。
    pub fn format0_glyph_index(&self, char_code: u8) -> u16 {
        if (char_code as usize) < self.glyph_index_array.len() {
            u16::from(self.glyph_index_array[char_code as usize])
        } else {
            0
        }
    }

    // ─── Format 4 ────────────────────────────────────────────────────────────

    /// 设置 Format 4 段映射数据。
    pub fn set_format4(
        &mut self,
        start_code: Vec<u16>,
        end_code: Vec<u16>,
        id_delta: Vec<i16>,
        id_range_offset: Vec<u16>,
        glyph_id_array: Vec<u16>,
    ) {
        self.start_code = start_code;
        self.end_code = end_code;
        self.id_delta = id_delta;
        self.id_range_offset = id_range_offset;
        self.glyph_id_array = glyph_id_array;
    }

    /// Format 4：通过字符编码获取字形索引。
    pub fn format4_glyph_index(&self, char_code: u16) -> u16 {
        for i in 0..self.start_code.len() {
            if self.end_code[i] < char_code {
                continue;
            }
            if self.start_code[i] > char_code {
                break;
            }

            if self.id_range_offset[i] == 0 {
                // 直接使用 id_delta
                return (i64::from(char_code) + i64::from(self.id_delta[i])) as u16;
            }
            // 通过 glyph_id_array 查找
            let offset = self.id_range_offset[i] / 2 + (char_code - self.start_code[i]) + i as u16
                - self.id_range_offset.len() as u16;
            let idx = offset as usize;
            if idx < self.glyph_id_array.len() {
                let mut glyph_id = self.glyph_id_array[idx];
                if glyph_id != 0 {
                    glyph_id = (i64::from(glyph_id) + i64::from(self.id_delta[i])) as u16;
                }
                return glyph_id;
            }
            break;
        }
        0
    }

    // ─── 通用接口 ────────────────────────────────────────────────────────────

    /// 通过字符编码获取字形索引（根据 format 自动选择算法）。
    pub fn glyph_index(&self, char_code: u32) -> u16 {
        match self.format {
            0 => self.format0_glyph_index(char_code as u8),
            4 => self.format4_glyph_index(char_code as u16),
            _ => 0,
        }
    }

    // ─── getter ──────────────────────────────────────────────────────────────

    /// 返回平台 ID。
    pub fn platform_id(&self) -> u16 {
        self.platform_id
    }
    /// 返回编码 ID。
    pub fn encoding_id(&self) -> u16 {
        self.encoding_id
    }
    /// 返回子表格式编号。
    pub fn format(&self) -> u16 {
        self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format0() {
        let mut sub = CmapSubtable::new(3, 1, 0);
        let mut arr = vec![0u8; 256];
        arr[65] = 36; // 'A' → glyph 36
        arr[66] = 37; // 'B' → glyph 37
        sub.set_format0(arr);
        assert_eq!(sub.glyph_index(65), 36);
        assert_eq!(sub.glyph_index(66), 37);
        assert_eq!(sub.glyph_index(67), 0);
    }

    #[test]
    fn test_format4_identity() {
        let mut sub = CmapSubtable::new(3, 1, 4);
        // 一个段：[0x41..0x41]，id_delta = 0
        sub.set_format4(vec![0x0041], vec![0x0041], vec![0], vec![0], vec![]);
        assert_eq!(sub.format4_glyph_index(0x41), 0x41);
        assert_eq!(sub.format4_glyph_index(0x42), 0);
    }

    #[test]
    fn test_format4_with_delta() {
        let mut sub = CmapSubtable::new(3, 1, 4);
        // 段 [0x41..0x43]，id_delta = 100
        sub.set_format4(vec![0x0041], vec![0x0043], vec![100], vec![0], vec![]);
        assert_eq!(sub.format4_glyph_index(0x41), 165); // 0x41 + 100
        assert_eq!(sub.format4_glyph_index(0x43), 167); // 0x43 + 100
    }

    #[test]
    fn test_glyph_index_dispatches() {
        let mut sub = CmapSubtable::new(3, 1, 0);
        let mut arr = vec![0u8; 256];
        arr[97] = 10; // 'a' → glyph 10
        sub.set_format0(arr);
        assert_eq!(sub.glyph_index(97), 10);
    }

    #[test]
    fn test_unsupported_format() {
        let sub = CmapSubtable::new(3, 1, 6);
        assert_eq!(sub.glyph_index(65), 0);
    }

    #[test]
    fn test_accessors() {
        let sub = CmapSubtable::new(3, 1, 4);
        assert_eq!(sub.platform_id(), 3);
        assert_eq!(sub.encoding_id(), 1);
        assert_eq!(sub.format(), 4);
    }
}
