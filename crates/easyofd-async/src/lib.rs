//! # easyofd-async
//!
//! easyofd 的异步门面，通过 `tokio::task::spawn_blocking` 将阻塞式 OFD 操作桥接到异步上下文。
//!
//! ## 方法学声明
//!
//! 本 crate **不是**原生异步 I/O 实现。OFD 读写涉及 ZIP 解压/压缩和 XML 解析，
//! 底层使用同步 `std::fs` 和 `std::io`，因此每个异步函数内部通过
//! `tokio::task::spawn_blocking` 将阻塞操作转移到 Tokio 的阻塞线程池执行。
//!
//! 这意味着：
//! - 调用方在 `async` 上下文中可以 `.await` 而不阻塞当前线程。
//! - CPU 密集的解析/序列化工作由 Tokio 线程池调度。
//! - 每次调用都会 spawn 一个 blocking task，对高频小文件场景有一定开销。
//!
//! ## 限制
//!
//! - 底层 `easyofd` 操作为同步阻塞，本 crate 仅提供异步包装。
//! - `spawn_blocking` 要求 Tokio 运行时已启动（`#[tokio::main]` 或手动创建）。
//! - 不支持 `WASM` 目标（`spawn_blocking` 在 WASM 上不可用）。
//!
//! ## 示例
//!
//! ```rust,ignore
//! use easyofd_async::{read_pages, write_pages_to_file};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 读取 OFD 文件
//!     let pages = read_pages("input.ofd").await?;
//!     println!("页数: {}", pages.len());
//!
//!     // 写入新 OFD 文件
//!     write_pages_to_file("output.ofd", pages).await?;
//!     Ok(())
//! }
//! ```

use easyofd::{EasyOfd, OfdError, OfdMetadata, OfdPage, OfdResult};
use std::path::{Path, PathBuf};

/// 读取 OFD 文件的所有页面。
///
/// 通过 `spawn_blocking` 在阻塞线程中执行文件 I/O 和 OFD 解析。
/// 内部将路径转为 `PathBuf` 以满足 `spawn_blocking` 的 `'static` 要求，
/// 调用方可传入 `&str`、`String`、`&Path`、`PathBuf` 等。
///
/// # Errors
///
/// 文件不存在、格式无效或解析失败时返回错误。
pub async fn read_pages(path: impl AsRef<Path> + Send) -> OfdResult<Vec<OfdPage>> {
    let path: PathBuf = path.as_ref().to_path_buf();
    tokio::task::spawn_blocking(move || EasyOfd::read(&path).map(|r| r.pages().to_vec()))
        .await
        .map_err(|e| OfdError::Io(std::io::Error::other(format!("task join error: {e}"))))?
}

/// 将页面写入 OFD 文件。
///
/// 通过 `spawn_blocking` 在阻塞线程中执行 OFD 序列化和文件写入。
///
/// # Errors
///
/// 序列化失败或文件 I/O 错误时返回错误。
pub async fn write_pages_to_file(
    path: impl AsRef<Path> + Send,
    pages: Vec<OfdPage>,
) -> OfdResult<()> {
    let path: PathBuf = path.as_ref().to_path_buf();
    tokio::task::spawn_blocking(move || EasyOfd::write_pages_to(&path, pages))
        .await
        .map_err(|e| OfdError::Io(std::io::Error::other(format!("task join error: {e}"))))?
}

/// 将页面构建为 OFD 字节数组（无文件 I/O）。
///
/// 通过 `spawn_blocking` 在阻塞线程中执行 OFD 序列化。
///
/// # Errors
///
/// 序列化失败时返回错误。
pub async fn write_pages_to_bytes(pages: Vec<OfdPage>) -> OfdResult<Vec<u8>> {
    tokio::task::spawn_blocking(move || EasyOfd::write_pages_to_bytes(pages))
        .await
        .map_err(|e| OfdError::Io(std::io::Error::other(format!("task join error: {e}"))))?
}

/// 读取 OFD 文件的元数据（标题、作者、创建时间等）。
///
/// 通过 `spawn_blocking` 在阻塞线程中执行文件 I/O 和 OFD 解析。
///
/// # Errors
///
/// 文件不存在、格式无效或解析失败时返回错误。
pub async fn read_metadata(path: impl AsRef<Path> + Send) -> OfdResult<OfdMetadata> {
    let path: PathBuf = path.as_ref().to_path_buf();
    tokio::task::spawn_blocking(move || EasyOfd::read(&path).map(|r| r.metadata().clone()))
        .await
        .map_err(|e| OfdError::Io(std::io::Error::other(format!("task join error: {e}"))))?
}

/// 将 OFD 文件转换为 Markdown 文本。
///
/// 通过 `spawn_blocking` 在阻塞线程中执行 OFD 解析和 Markdown 转换。
///
/// # Errors
///
/// 文件不存在、格式无效或转换失败时返回错误。
pub async fn to_markdown(
    path: impl AsRef<Path> + Send,
) -> OfdResult<easyofd::MarkdownConversionResult> {
    let path: PathBuf = path.as_ref().to_path_buf();
    tokio::task::spawn_blocking(move || EasyOfd::to_markdown(&path).do_convert())
        .await
        .map_err(|e| OfdError::Io(std::io::Error::other(format!("task join error: {e}"))))?
}

/// 验证 OFD 文件的数字签名（GB/T 38540）。
///
/// 通过 `spawn_blocking` 在阻塞线程中执行签名验证。
///
/// # Errors
///
/// 文件不存在、签名结构无效或密码学验证失败时返回错误。
/// 返回 `Ok(true)` 表示签名有效，`Ok(false)` 表示签名无效。
pub async fn verify(path: impl AsRef<Path> + Send) -> OfdResult<bool> {
    let path: PathBuf = path.as_ref().to_path_buf();
    tokio::task::spawn_blocking(move || easyofd::verify_signature(&path))
        .await
        .map_err(|e| OfdError::Io(std::io::Error::other(format!("task join error: {e}"))))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd::TextObject;

    /// 测试 read -> write roundtrip。
    #[tokio::test]
    async fn test_read_write_roundtrip() {
        let dir = std::env::temp_dir().join("easyofd_async_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("async_roundtrip.ofd");

        // 先用同步方式写入测试文件
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "异步测试文本"));
        EasyOfd::write_pages_to(&path, vec![page]).unwrap();

        // 异步读取
        let pages = read_pages(&path).await.unwrap();
        assert_eq!(pages.len(), 1);

        // 异步写入新文件
        let out_path = dir.join("async_roundtrip_out.ofd");
        write_pages_to_file(&out_path, pages).await.unwrap();

        // 验证输出文件
        let bytes = std::fs::read(&out_path).unwrap();
        assert_eq!(&bytes[0..2], b"PK");

        let _ = std::fs::remove_dir_all(dir);
    }

    /// 测试 read_metadata。
    #[tokio::test]
    async fn test_read_metadata() {
        let dir = std::env::temp_dir().join("easyofd_async_meta");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("meta.ofd");

        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "metadata test"));
        EasyOfd::write_pages(path.to_string_lossy().into_owned())
            .metadata_title("Async 测试标题")
            .do_write(vec![page])
            .unwrap();

        let meta = read_metadata(&path).await.unwrap();
        assert_eq!(meta.title.as_deref(), Some("Async 测试标题"));

        let _ = std::fs::remove_dir_all(dir);
    }

    /// 测试 to_markdown。
    #[tokio::test]
    async fn test_to_markdown() {
        let dir = std::env::temp_dir().join("easyofd_async_md");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("md.ofd");

        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 10.0, "Markdown 转换测试"));
        EasyOfd::write_pages_to(&path, vec![page]).unwrap();

        let result = to_markdown(&path).await.unwrap();
        assert!(result.markdown.contains("Markdown 转换测试"));
        assert_eq!(result.report.pages_converted, 1);

        let _ = std::fs::remove_dir_all(dir);
    }

    /// 测试 write_pages_to_bytes。
    #[tokio::test]
    async fn test_write_pages_to_bytes() {
        let page = OfdPage::new(210.0, 297.0);
        let bytes = write_pages_to_bytes(vec![page]).await.unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    /// 测试错误路径：无效文件。
    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let result = read_pages("/nonexistent/path/file.ofd").await;
        assert!(result.is_err());
    }
}
