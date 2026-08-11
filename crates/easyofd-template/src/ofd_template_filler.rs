//! OFD 模板填充器。

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

use easyofd_core::OfdResult;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// OFD 模板填充器。
///
/// 打开模板 OFD 文件，替换 XML 内容中的 `{key}` 占位符，
/// 并将结果写入新的 OFD 文件。
pub struct OfdTemplateFiller {
    output: Vec<u8>,
}

impl OfdTemplateFiller {
    /// 使用占位符值填充模板 OFD。
    ///
    /// 支持所有 XML 文件中的 `{key}` 风格占位符。
    ///
    /// # 错误
    ///
    /// 模板文件无法读取或不是有效 ZIP 时返回错误。
    pub fn fill(
        template_path: impl AsRef<std::path::Path>,
        data: &HashMap<String, String>,
    ) -> OfdResult<Self> {
        let template_bytes = std::fs::read(template_path).map_err(easyofd_core::OfdError::Io)?;
        Self::fill_bytes(&template_bytes, data)
    }

    /// 从内存字节数组填充模板 OFD。
    ///
    /// # 错误
    ///
    /// 数据不是有效 ZIP 时返回错误。
    pub fn fill_bytes(template_bytes: &[u8], data: &HashMap<String, String>) -> OfdResult<Self> {
        let cursor = Cursor::new(template_bytes);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        easyofd_package::validate_archive(&mut archive, easyofd_package::PackageLimits::default())?;

        let out_buf = Vec::new();
        let out_cursor = Cursor::new(out_buf);
        let mut zip = ZipWriter::new(out_cursor);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            let name = entry.name().to_string();
            let mut content = Vec::new();
            entry
                .read_to_end(&mut content)
                .map_err(easyofd_core::OfdError::Io)?;

            // 替换 XML 文件中的占位符
            let is_xml = std::path::Path::new(&name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"));
            if is_xml {
                let text = String::from_utf8(content)
                    .map_err(|error| easyofd_core::OfdError::Xml(format!("{name}: {error}")))?;
                let mut replaced = text;
                for (key, value) in data {
                    let placeholder = format!("{{{key}}}");
                    replaced = replaced.replace(&placeholder, &xml_escape(value));
                }
                zip.start_file(name, options)
                    .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
                zip.write_all(replaced.as_bytes())
                    .map_err(easyofd_core::OfdError::Io)?;
            } else {
                // 二进制文件（图片等）— 原样复制
                zip.start_file(name, options)
                    .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
                zip.write_all(&content)
                    .map_err(easyofd_core::OfdError::Io)?;
            }
        }

        let cursor = zip
            .finish()
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        let output = cursor.into_inner();

        Ok(Self { output })
    }

    /// 返回填充后的 OFD 字节。
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.output
    }

    /// 将填充后的 OFD 保存到文件。
    ///
    /// # 错误
    ///
    /// 文件 I/O 失败时返回错误。
    pub fn save(self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        easyofd_package::atomic_write(path, |file| {
            file.write_all(&self.output)?;
            Ok(())
        })
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
