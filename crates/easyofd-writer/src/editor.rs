//! OFD 编辑器 — 打开、修改、保存已有 OFD 文件。

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use easyofd_core::{OfdError, OfdMetadata, OfdPage, OfdResult, TextObject, Watermark};

use crate::{OfdWriter, WriteOptions};

/// OFD 编辑器。打开已有 OFD 文件，支持添加文本、页面和水印，然后保存。
#[allow(dead_code)]
pub struct OfdEditor {
    pages: Vec<OfdPage>,
    metadata: OfdMetadata,
    original_entries: HashMap<String, Vec<u8>>,
}

impl OfdEditor {
    /// 打开已有 OFD 文件进行编辑。
    ///
    /// # Errors
    ///
    /// 文件不存在或不是有效 OFD 时返回错误。
    pub fn open(path: impl Into<String>) -> OfdResult<Self> {
        let path = path.into();
        let bytes = std::fs::read(&path).map_err(OfdError::Io)?;
        Self::from_bytes(&bytes)
    }

    /// 从字节打开 OFD 进行编辑。
    ///
    /// # Errors
    ///
    /// 不是有效 ZIP 或 OFD 时返回错误。
    pub fn from_bytes(bytes: &[u8]) -> OfdResult<Self> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| OfdError::Zip(e.to_string()))?;

        let mut original_entries = HashMap::new();
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| OfdError::Zip(e.to_string()))?;
            let name = file.name().to_string();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(OfdError::Io)?;
            original_entries.insert(name, buf);
        }

        // 解析页面（简化：从原始 Reader 复用）
        let reader = easyofd_reader::OfdReader::from_bytes(bytes)?;
        let pages: Vec<OfdPage> = reader.pages().to_vec();

        Ok(Self {
            pages,
            metadata: OfdMetadata::default(),
            original_entries,
        })
    }

    /// 页面数量。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// 向指定页面添加文本。
    ///
    /// # Errors
    ///
    /// 页面索引越界时返回错误。
    pub fn add_text_to_page(&mut self, page_index: usize, text: TextObject) -> OfdResult<()> {
        let page = self.pages.get_mut(page_index).ok_or_else(|| {
            OfdError::InvalidDocument(format!("Page index {page_index} out of range"))
        })?;
        page.add_text(text);
        Ok(())
    }

    /// 添加新页面。
    pub fn add_page(&mut self, page: OfdPage) {
        self.pages.push(page);
    }

    /// 应用水印到所有页面。
    pub fn apply_watermarks(&mut self, watermarks: &[Watermark]) {
        for wm in watermarks {
            for (i, page) in self.pages.iter_mut().enumerate() {
                let target = wm.page.is_none_or(|p| p == i + 1);
                if target {
                    page.add_text(TextObject::new(
                        wm.position.0,
                        wm.position.1,
                        wm.text.as_deref().unwrap_or(""),
                    ));
                }
            }
        }
    }

    /// 保存编辑后的 OFD 到文件。
    ///
    /// # Errors
    ///
    /// ZIP 创建或文件写入失败时返回错误。
    pub fn save(&self, path: impl AsRef<Path>) -> OfdResult<()> {
        let mut writer = OfdWriter::with_options(WriteOptions {
            metadata: self.metadata.clone(),
        });
        writer.add_pages(self.pages.clone());
        writer.build_to_file(path)
    }

    /// 保存编辑后的 OFD 到字节。
    ///
    /// # Errors
    ///
    /// ZIP 创建失败时返回错误。
    pub fn save_to_bytes(&self) -> OfdResult<Vec<u8>> {
        let mut writer = OfdWriter::with_options(WriteOptions {
            metadata: self.metadata.clone(),
        });
        writer.add_pages(self.pages.clone());
        writer.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{TextObject, Watermark};

    fn make_test_ofd() -> Vec<u8> {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "Original text"));
        let mut w = OfdWriter::new();
        w.add_page(page);
        w.build().unwrap()
    }

    #[test]
    fn test_editor_open_and_save() {
        let bytes = make_test_ofd();
        let dir = std::env::temp_dir().join("easyofd_editor_split");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ofd");
        std::fs::write(&path, &bytes).unwrap();

        let mut editor = OfdEditor::open(path.to_string_lossy().into_owned()).unwrap();
        assert_eq!(editor.page_count(), 1);

        editor
            .add_text_to_page(0, TextObject::new(10.0, 40.0, "Edited text"))
            .unwrap();

        let out = dir.join("edited.ofd");
        editor.save(&out).unwrap();

        let reader = easyofd_reader::OfdReader::open(&out).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("Original text"));
        assert!(text.contains("Edited text"));

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_editor_append_page() {
        let bytes = make_test_ofd();
        let dir = std::env::temp_dir().join("easyofd_editor_split2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ofd");
        std::fs::write(&path, &bytes).unwrap();

        let mut editor = OfdEditor::open(path.to_string_lossy().into_owned()).unwrap();
        let mut new_page = OfdPage::new(210.0, 297.0);
        new_page.add_text(TextObject::new(10.0, 20.0, "Page 2"));
        editor.add_page(new_page);
        assert_eq!(editor.page_count(), 2);

        let out = dir.join("two_pages.ofd");
        editor.save(&out).unwrap();

        let reader = easyofd_reader::OfdReader::open(&out).unwrap();
        assert_eq!(reader.page_count(), 2);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_editor_watermark() {
        let bytes = make_test_ofd();
        let dir = std::env::temp_dir().join("easyofd_editor_split3");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ofd");
        std::fs::write(&path, &bytes).unwrap();

        let mut editor = OfdEditor::open(path.to_string_lossy().into_owned()).unwrap();
        editor.apply_watermarks(&[Watermark {
            text: Some("CONFIDENTIAL".into()),
            position: (50.0, 150.0),
            ..Watermark::default()
        }]);

        let out = dir.join("watermarked.ofd");
        editor.save(&out).unwrap();

        let reader = easyofd_reader::OfdReader::open(&out).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("CONFIDENTIAL"));
        assert!(text.contains("Original text"));

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_editor_invalid_page() {
        let bytes = make_test_ofd();
        let dir = std::env::temp_dir().join("easyofd_editor_split4");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ofd");
        std::fs::write(&path, &bytes).unwrap();

        let mut editor = OfdEditor::open(path.to_string_lossy().into_owned()).unwrap();
        let result = editor.add_text_to_page(99, TextObject::new(0.0, 0.0, "x"));
        assert!(result.is_err());

        let _ = std::fs::remove_file(&path);
    }
}
