//! OFD 包安全边界与原子文件输出。
//!
//! 该模块集中处理 ZIP 炸弹限制、路径校验和同目录原子替换，避免 Reader、
//! Writer、Template 各自实现不一致的安全策略。

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use easyofd_core::{OfdError, OfdResult};

/// OFD 包的资源限制。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageLimits {
    /// ZIP 条目数量上限。
    pub max_entries: usize,
    /// 所有解压条目的总字节数上限。
    pub max_total_uncompressed_size: u64,
    /// 单个条目的解压字节数上限。
    pub max_entry_uncompressed_size: u64,
    /// 最大压缩比，阻止高压缩率 ZIP 炸弹。
    pub max_compression_ratio: u64,
}

impl Default for PackageLimits {
    fn default() -> Self {
        Self {
            max_entries: 20_000,
            max_total_uncompressed_size: 1_073_741_824,
            max_entry_uncompressed_size: 268_435_456,
            max_compression_ratio: 1_000,
        }
    }
}

/// 校验 ZIP 中所有条目的路径和资源占用。
///
/// # Errors
///
/// 当条目数量、解压大小、压缩比或路径超过安全限制时返回错误。
pub fn validate_archive<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    limits: PackageLimits,
) -> OfdResult<()> {
    if archive.len() > limits.max_entries {
        return Err(OfdError::InvalidDocument(format!(
            "ZIP entry count {} exceeds limit {}",
            archive.len(),
            limits.max_entries
        )));
    }

    let mut total = 0_u64;
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|error| OfdError::Zip(error.to_string()))?;
        validate_entry_name(file.name())?;
        let size = file.size();
        if size > limits.max_entry_uncompressed_size {
            return Err(OfdError::InvalidDocument(format!(
                "ZIP entry {} is too large: {size} bytes",
                file.name()
            )));
        }
        total = total.checked_add(size).ok_or_else(|| {
            OfdError::InvalidDocument("ZIP uncompressed size overflow".to_string())
        })?;
        if total > limits.max_total_uncompressed_size {
            return Err(OfdError::InvalidDocument(format!(
                "ZIP uncompressed size {total} exceeds limit {}",
                limits.max_total_uncompressed_size
            )));
        }

        let compressed = file.compressed_size();
        if size > 0 && (compressed == 0 || size / compressed.max(1) > limits.max_compression_ratio)
        {
            return Err(OfdError::InvalidDocument(format!(
                "ZIP entry {} exceeds compression ratio limit",
                file.name()
            )));
        }
    }
    Ok(())
}

/// 校验 ZIP 条目为包内相对路径。
///
/// # Errors
///
/// 绝对路径、父目录跳转及平台前缀均被拒绝。
pub fn validate_entry_name(name: &str) -> OfdResult<()> {
    if name.is_empty() || name.contains('\\') {
        return Err(OfdError::InvalidDocument(format!(
            "unsafe ZIP entry path: {name}"
        )));
    }
    let path = Path::new(name);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(OfdError::InvalidDocument(format!(
            "unsafe ZIP entry path: {name}"
        )));
    }
    Ok(())
}

/// 在目标文件同目录写入临时文件，成功后原子替换目标文件。
///
/// # Errors
///
/// 创建、写入或替换文件失败时返回 I/O 错误，并尽力清理临时文件。
pub fn atomic_write(
    target: impl AsRef<Path>,
    write: impl FnOnce(&mut File) -> OfdResult<()>,
) -> OfdResult<()> {
    let target = target.as_ref();
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let temporary = temporary_path(target);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        write(&mut file)?;
        file.sync_all()?;
        drop(file);
        replace_file(&temporary, target)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn temporary_path(target: &Path) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.ofd");
    target.with_file_name(format!(".{name}.{nonce}.tmp"))
}

fn replace_file(source: &Path, target: &Path) -> OfdResult<()> {
    match std::fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(_error) if target.exists() => {
            let backup = target.with_extension("easyofd-backup");
            std::fs::rename(target, &backup)?;
            match std::fs::rename(source, target) {
                Ok(()) => {
                    let _ = std::fs::remove_file(backup);
                    Ok(())
                }
                Err(rename_error) => {
                    let _ = std::fs::rename(&backup, target);
                    Err(OfdError::Io(rename_error))
                }
            }
        }
        Err(error) => Err(OfdError::Io(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_directory() {
        assert!(validate_entry_name("../secret").is_err());
        assert!(validate_entry_name("Doc_0/Pages/Page_0.xml").is_ok());
    }

    #[test]
    fn atomically_replaces_file() {
        let path = std::env::temp_dir().join("easyofd_atomic_write_test.ofd");
        std::fs::write(&path, b"old").unwrap();
        atomic_write(&path, |file| {
            use std::io::Write;
            file.write_all(b"new")?;
            Ok(())
        })
        .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn failed_write_preserves_original_file() {
        let path = std::env::temp_dir().join("easyofd_atomic_failure_test.ofd");
        std::fs::write(&path, b"original").unwrap();
        let result = atomic_write(&path, |_file| {
            Err(OfdError::InvalidDocument("simulated failure".into()))
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        let _ = std::fs::remove_file(path);
    }
}
