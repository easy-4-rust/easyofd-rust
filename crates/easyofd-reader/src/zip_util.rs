//! ZIP 文件解压工具。
//!
//! 对应 Java: org.ofdrw.reader.ZipUtil
//!
//! Java 版使用 Apache Commons Compress 解压 ZIP 文件到文件系统。
//! Rust 版提供内存解压和文件系统解压两种方式，底层使用 `zip` crate。

use easyofd_core::{OfdError, OfdResult};
use std::io::{Read, Seek};
use std::path::Path;

/// 从 ZIP 归档中读取指定条目的全部字节。
///
/// 对应 Java: `ZipUtil` 中的文件读取逻辑
///
/// # 错误
///
/// 条目不存在或读取失败时返回错误。
pub fn read_entry<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> OfdResult<Vec<u8>> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| OfdError::Zip(format!("{name}: {e}")))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(OfdError::Io)?;
    Ok(buf)
}

/// 列出 ZIP 归档中所有条目名称。
///
/// 对应 Java: `ZipUtil` 中的目录遍历逻辑
///
/// # 错误
///
/// 归档读取失败时返回错误。
pub fn list_entries<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<Vec<String>> {
    let mut names = Vec::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| OfdError::Zip(format!("index {i}: {e}")))?;
        names.push(file.name().to_string());
    }
    Ok(names)
}

/// 将 ZIP 归档解压到指定目录。
///
/// 对应 Java: `ZipUtil.unZipFileByApacheCommonCompress`
///
/// # 错误
///
/// IO 操作失败或归档格式错误时返回错误。
pub fn extract_to_dir<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    dest: &Path,
) -> OfdResult<()> {
    use std::fs;
    use std::io::Write;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| OfdError::Zip(format!("index {i}: {e}")))?;
        let outpath = dest.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(OfdError::Io)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(OfdError::Io)?;
            }
            let mut outfile = fs::File::create(&outpath).map_err(OfdError::Io)?;
            std::io::copy(&mut file, &mut outfile).map_err(OfdError::Io)?;
        }
    }
    Ok(())
}

/// 将内存中的 ZIP 字节解压到指定目录。
///
/// 对应 Java: `ZipUtil.unZipFiles(InputStream, String)`
///
/// # 错误
///
/// ZIP 格式错误或 IO 操作失败时返回错误。
pub fn extract_bytes_to_dir(data: &[u8], dest: &Path) -> OfdResult<()> {
    let reader = std::io::Cursor::new(data);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| OfdError::Zip(format!("{e}")))?;
    extract_to_dir(&mut archive, dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn create_test_zip() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let writer = Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(writer);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            zip.start_file("OFD.xml", options).unwrap();
            zip.write_all(b"<OFD/>").unwrap();

            zip.start_file("Doc_0/Document.xml", options).unwrap();
            zip.write_all(b"<Document/>").unwrap();

            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn test_read_entry() {
        let data = create_test_zip();
        let reader = Cursor::new(&data);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let content = read_entry(&mut archive, "OFD.xml").unwrap();
        assert_eq!(content, b"<OFD/>");
    }

    #[test]
    fn test_read_entry_not_found() {
        let data = create_test_zip();
        let reader = Cursor::new(&data);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let result = read_entry(&mut archive, "nonexistent.xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_entries() {
        let data = create_test_zip();
        let reader = Cursor::new(&data);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let entries = list_entries(&mut archive).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&"OFD.xml".to_string()));
        assert!(entries.contains(&"Doc_0/Document.xml".to_string()));
    }

    #[test]
    fn test_extract_to_dir() {
        let data = create_test_zip();
        let reader = Cursor::new(&data);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let dest = std::env::temp_dir().join("easyofd_zip_util_test");
        let _ = std::fs::remove_dir_all(&dest);
        std::fs::create_dir_all(&dest).unwrap();

        extract_to_dir(&mut archive, &dest).unwrap();

        assert!(dest.join("OFD.xml").exists());
        assert!(dest.join("Doc_0/Document.xml").exists());

        let content = std::fs::read_to_string(dest.join("OFD.xml")).unwrap();
        assert_eq!(content, "<OFD/>");

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn test_extract_bytes_to_dir() {
        let data = create_test_zip();
        let dest = std::env::temp_dir().join("easyofd_zip_util_bytes_test");
        let _ = std::fs::remove_dir_all(&dest);
        std::fs::create_dir_all(&dest).unwrap();

        extract_bytes_to_dir(&data, &dest).unwrap();
        assert!(dest.join("OFD.xml").exists());

        let _ = std::fs::remove_dir_all(&dest);
    }
}
