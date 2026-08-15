//! # easyofd-ffi
//!
//! C ABI 绑定，供 C/C++/Python/其他语言嵌入 easyofd OFD 文档操作。
//!
//! ## 用途
//!
//! 提供最小化 C 函数接口，允许外部语言通过 `extern "C"` 调用 OFD 读写操作。
//!
//! ## 限制
//!
//! - 每个 reader/writer 实例非线程安全，不可并发使用同一实例。
//! - 所有字符串采用 UTF-8 编码，返回的 `char*` 由 Rust `CString` 分配，
//!   调用方必须用 `easyofd_string_free()` 释放。
//! - 本 crate 使用 `unsafe`（FFI 边界必需），所有 unsafe 块均带 SAFETY 注释。
//!
//! ## 示例（C 语言）
//!
//! ```c
//! #include "easyofd_ffi.h"
//!
//! // 读取 OFD
//! easyofd_reader* reader = easyofd_reader_from_file("input.ofd");
//! if (!reader) { /* 通过 easyofd_last_error() 获取错误 */ }
//! int32_t pages = easyofd_page_count(reader);
//! // ... 使用完毕
//! easyofd_reader_free(reader);
//!
//! // 写入 OFD
//! easyofd_writer* writer = easyofd_writer_new();
//! easyofd_writer_add_text_page(writer, 210.0, 297.0, 20.0, 30.0, "Hello!");
//! easyofd_writer_build_to_file(writer, "output.ofd");
//! easyofd_writer_free(writer);
//! ```

// FFI 边界需要 i32/usize 互转，这些 cast 在 OFD 页数范围内是安全的。
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

use easyofd::{EasyOfd, OfdPage, OfdReader, OfdWriter, TextObject};

// ─── 线程局部错误存储 ─────────────────────────────────────────────────────────

thread_local! {
    static LAST_ERROR: Mutex<Option<CString>> = const { Mutex::new(None) };
}

/// 设置线程局部错误信息。
fn set_last_error(msg: &str) {
    // SAFETY: CString::new 仅在包含 null 字节时失败，错误信息不会含 null。
    let c = CString::new(msg).unwrap_or_else(|_| CString::new("unknown error").unwrap());
    LAST_ERROR.with(|cell| {
        if let Ok(mut guard) = cell.lock() {
            *guard = Some(c);
        }
    });
}

/// 清除线程局部错误信息。
fn clear_last_error() {
    LAST_ERROR.with(|cell| {
        if let Ok(mut guard) = cell.lock() {
            *guard = None;
        }
    });
}

// ─── 不透明句柄 ───────────────────────────────────────────────────────────────

/// OFD 文档读取器不透明句柄。
///
/// # Safety
///
/// 仅通过 FFI 函数访问，内部持有 `OfdReader`。
#[allow(non_camel_case_types)]
pub struct easyofd_reader {
    inner: OfdReader,
}

/// OFD 文档写入器不透明句柄。
///
/// # Safety
///
/// 仅通过 FFI 函数访问，内部持有 `OfdWriter`。
#[allow(non_camel_case_types)]
pub struct easyofd_writer {
    inner: OfdWriter,
}

// ─── 辅助：C 字符串 → Rust &str ───────────────────────────────────────────────

/// 将 C 字符串指针转换为 Rust &str。
///
/// # Safety
///
/// `ptr` 必须是指向以 null 结尾的有效 UTF-8 字符串的指针，或为 NULL。
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: 调用方保证 ptr 指向有效的 null 结尾字符串。
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

// ─── Reader API ───────────────────────────────────────────────────────────────

/// 从文件路径创建读取器。
///
/// # Safety
///
/// `path` 必须是指向以 null 结尾的有效 UTF-8 文件路径的指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_reader_from_file(path: *const c_char) -> *mut easyofd_reader {
    clear_last_error();
    // SAFETY: 调用方保证 path 指向有效的 null 结尾字符串。
    let Some(path_str) = (unsafe { cstr_to_str(path) }) else {
        set_last_error("path is NULL or not valid UTF-8");
        return ptr::null_mut();
    };
    match EasyOfd::read(path_str) {
        Ok(reader) => Box::into_raw(Box::new(easyofd_reader { inner: reader })),
        Err(e) => {
            set_last_error(&format!("{e}"));
            ptr::null_mut()
        }
    }
}

/// 从内存字节创建读取器。
///
/// # Safety
///
/// `data` 必须指向至少 `len` 字节的有效内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_reader_from_bytes(
    data: *const u8,
    len: usize,
) -> *mut easyofd_reader {
    clear_last_error();
    if data.is_null() {
        set_last_error("data pointer is NULL");
        return ptr::null_mut();
    }
    // SAFETY: 调用方保证 data 指向至少 len 字节的有效内存。
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    match EasyOfd::read_from_bytes(slice) {
        Ok(reader) => Box::into_raw(Box::new(easyofd_reader { inner: reader })),
        Err(e) => {
            set_last_error(&format!("{e}"));
            ptr::null_mut()
        }
    }
}

/// 释放读取器句柄。传入 NULL 安全（无操作）。
///
/// # Safety
///
/// `reader` 必须是由 `easyofd_reader_from_file` 或 `easyofd_reader_from_bytes`
/// 返回的有效指针，或为 NULL。释放后不可再使用该指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_reader_free(reader: *mut easyofd_reader) {
    if !reader.is_null() {
        // SAFETY: 调用方保证 reader 是由 create 函数返回的有效指针，
        // 且释放后不再使用。
        drop(unsafe { Box::from_raw(reader) });
    }
}

/// 获取文档页数。
///
/// # Safety
///
/// `reader` 必须是由 create 函数返回的有效非空指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_page_count(reader: *const easyofd_reader) -> i32 {
    if reader.is_null() {
        return 0;
    }
    // SAFETY: 调用方保证 reader 非空且有效。
    let r = unsafe { &*reader };
    // OFD 文档页数不会超过 i32::MAX。
    r.inner.page_count() as i32
}

/// 提取指定页面的文本内容，写入调用方提供的数组。
///
/// # Safety
///
/// - `reader` 必须是由 create 函数返回的有效非空指针。
/// - `out` 必须指向至少 `cap` 个 `char*` 槽位的有效内存。
/// - 返回后 `out` 中每个非空字符串由 Rust 分配，调用方须用 `easyofd_string_free()` 释放。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_page_texts(
    reader: *const easyofd_reader,
    page: i32,
    out: *mut *mut c_char,
    cap: i32,
) -> i32 {
    clear_last_error();
    if reader.is_null() {
        set_last_error("reader is NULL");
        return -1;
    }
    if out.is_null() || cap < 0 {
        set_last_error("out buffer is NULL or cap is negative");
        return -1;
    }
    // SAFETY: 调用方保证 reader 非空且有效。
    let r = unsafe { &*reader };
    let page_idx = page as usize;
    if page_idx == 0 || page_idx > r.inner.page_count() {
        set_last_error(&format!(
            "page {} out of range (1..={})",
            page,
            r.inner.page_count()
        ));
        return -1;
    }
    let pages = r.inner.pages();
    let page_data = &pages[page_idx - 1];
    let texts: Vec<&str> = page_data
        .content
        .iter()
        .filter_map(|obj| {
            if let easyofd::ContentObject::Text(t) = obj {
                Some(t.text.as_str())
            } else {
                None
            }
        })
        .collect();
    let count = texts.len().min(cap as usize);
    for i in 0..count {
        // SAFETY: 调用方保证 out 指向至少 cap 个槽位，且 i < cap。
        unsafe {
            *out.add(i) = CString::new(texts[i])
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw();
        }
    }
    count as i32
}

/// 提取指定页面的全部文本合并为一个字符串。
///
/// # Safety
///
/// `reader` 必须是由 create 函数返回的有效非空指针。
/// 返回的字符串由 Rust 分配，调用方须用 `easyofd_string_free()` 释放。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_page_text_merged(
    reader: *const easyofd_reader,
    page: i32,
) -> *mut c_char {
    clear_last_error();
    if reader.is_null() {
        set_last_error("reader is NULL");
        return ptr::null_mut();
    }
    // SAFETY: 调用方保证 reader 非空且有效。
    let r = unsafe { &*reader };
    let page_idx = page as usize;
    if page_idx == 0 || page_idx > r.inner.page_count() {
        set_last_error(&format!(
            "page {} out of range (1..={})",
            page,
            r.inner.page_count()
        ));
        return ptr::null_mut();
    }
    let pages = r.inner.pages();
    let merged: String = pages[page_idx - 1]
        .content
        .iter()
        .filter_map(|obj| {
            if let easyofd::ContentObject::Text(t) = obj {
                Some(t.text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    CString::new(merged)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

/// 提取全部页面的文本（以 "---" 分隔）。
///
/// # Safety
///
/// `reader` 必须是由 create 函数返回的有效非空指针。
/// 返回的字符串由 Rust 分配，调用方须用 `easyofd_string_free()` 释放。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_extract_all_text(reader: *const easyofd_reader) -> *mut c_char {
    clear_last_error();
    if reader.is_null() {
        set_last_error("reader is NULL");
        return ptr::null_mut();
    }
    // SAFETY: 调用方保证 reader 非空且有效。
    let r = unsafe { &*reader };
    let all = r.inner.extract_all_text();
    CString::new(all)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

// ─── Writer API ───────────────────────────────────────────────────────────────

/// 创建空的写入器。
#[unsafe(no_mangle)]
pub extern "C" fn easyofd_writer_new() -> *mut easyofd_writer {
    clear_last_error();
    Box::into_raw(Box::new(easyofd_writer {
        inner: OfdWriter::new(),
    }))
}

/// 向写入器添加文本页。
///
/// # Safety
///
/// - `writer` 必须是由 `easyofd_writer_new` 返回的有效非空指针。
/// - `text` 必须指向以 null 结尾的有效 UTF-8 字符串。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_writer_add_text_page(
    writer: *mut easyofd_writer,
    width: f64,
    height: f64,
    text_x: f64,
    text_y: f64,
    text: *const c_char,
) -> i32 {
    clear_last_error();
    if writer.is_null() {
        set_last_error("writer is NULL");
        return -1;
    }
    // SAFETY: 调用方保证 writer 非空且有效。
    let w = unsafe { &mut *writer };
    // SAFETY: 调用方保证 text 指向有效的 null 结尾字符串。
    let Some(text_str) = (unsafe { cstr_to_str(text) }) else {
        set_last_error("text is NULL or not valid UTF-8");
        return -1;
    };
    let mut page = OfdPage::new(width, height);
    page.add_text(TextObject::new(text_x, text_y, text_str));
    w.inner.add_page(page);
    0
}

/// 将写入器内容构建为 OFD 文件并写入磁盘。
///
/// # Safety
///
/// - `writer` 必须是由 `easyofd_writer_new` 返回的有效非空指针。
/// - `path` 必须指向以 null 结尾的有效 UTF-8 文件路径。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_writer_build_to_file(
    writer: *const easyofd_writer,
    path: *const c_char,
) -> i32 {
    clear_last_error();
    if writer.is_null() {
        set_last_error("writer is NULL");
        return -1;
    }
    // SAFETY: 调用方保证 writer 非空且有效。
    let w = unsafe { &*writer };
    // SAFETY: 调用方保证 path 指向有效的 null 结尾字符串。
    let Some(path_str) = (unsafe { cstr_to_str(path) }) else {
        set_last_error("path is NULL or not valid UTF-8");
        return -1;
    };
    match w.inner.build_to_file(path_str) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(&format!("{e}"));
            -1
        }
    }
}

/// 将写入器内容构建为 OFD 字节。
///
/// # Safety
///
/// - `writer` 必须是由 `easyofd_writer_new` 返回的有效非空指针。
/// - `out_len` 必须指向有效的 `size_t` 内存。
/// - 返回的字节数组由 Rust 分配，调用方须用 `easyofd_bytes_free()` 释放。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_writer_build_to_bytes(
    writer: *const easyofd_writer,
    out_len: *mut usize,
) -> *mut u8 {
    clear_last_error();
    if writer.is_null() {
        set_last_error("writer is NULL");
        return ptr::null_mut();
    }
    // SAFETY: 调用方保证 writer 非空且有效。
    let w = unsafe { &*writer };
    match w.inner.build() {
        Ok(mut bytes) => {
            // shrink_to_fit 确保 capacity == len，这样 bytes_free 可以用
            // Vec::from_raw_parts(ptr, len, len) 正确释放。
            bytes.shrink_to_fit();
            let len = bytes.len();
            assert_eq!(len, bytes.capacity());
            // SAFETY: 调用方保证 out_len 指向有效内存。
            unsafe {
                *out_len = len;
            }
            let ptr = bytes.as_ptr();
            // 泄漏 Vec，将所有权交给 C 调用方。bytes_free 会恢复 Vec 并 drop。
            std::mem::forget(bytes);
            ptr.cast_mut()
        }
        Err(e) => {
            set_last_error(&format!("{e}"));
            ptr::null_mut()
        }
    }
}

/// 释放写入器句柄。传入 NULL 安全（无操作）。
///
/// # Safety
///
/// `writer` 必须是由 `easyofd_writer_new` 返回的有效指针，或为 NULL。
/// 释放后不可再使用该指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_writer_free(writer: *mut easyofd_writer) {
    if !writer.is_null() {
        // SAFETY: 调用方保证 writer 是由 create 函数返回的有效指针，
        // 且释放后不再使用。
        drop(unsafe { Box::from_raw(writer) });
    }
}

// ─── 错误与内存管理 ──────────────────────────────────────────────────────────

/// 获取最近一次 FFI 调用的错误信息。
///
/// # Safety
///
/// 返回的字符串由 Rust 分配，调用方须用 `easyofd_string_free()` 释放。
/// 返回 NULL 表示无错误。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_last_error() -> *mut c_char {
    LAST_ERROR.with(|cell| {
        if let Ok(guard) = cell.lock() {
            guard
                .as_ref()
                .map_or(ptr::null_mut(), |c| c.clone().into_raw())
        } else {
            ptr::null_mut()
        }
    })
}

/// 释放 Rust 分配的字符串。传入 NULL 安全（无操作）。
///
/// # Safety
///
/// `s` 必须是由 FFI 函数返回的 Rust 分配字符串指针，或为 NULL。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_string_free(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: 调用方保证 s 是由 CString::into_raw 返回的有效指针。
        drop(unsafe { CString::from_raw(s) });
    }
}

/// 释放 Rust 分配的字节数组。传入 NULL 安全（无操作）。
///
/// # Safety
///
/// `data` 必须是由 `easyofd_writer_build_to_bytes` 返回的有效指针，或为 NULL。
/// `len` 必须与 `build_to_bytes` 输出的 `out_len` 一致。
#[allow(clippy::same_length_and_capacity)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn easyofd_bytes_free(data: *mut u8, len: usize) {
    if !data.is_null() && len > 0 {
        // SAFETY: 调用方保证 data 是由 Vec（shrink_to_fit 后）泄漏的指针，
        // len 与泄漏时一致。shrink_to_fit 保证 capacity == len，
        // 因此 Vec::from_raw_parts(ptr, len, len) 是正确的。
        drop(unsafe { Vec::from_raw_parts(data, len, len) });
    }
}

// ─── Rust 侧集成测试 ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 roundtrip：创建 writer -> 添加页 -> build -> reader -> 读取文本。
    #[test]
    fn test_roundtrip_file() {
        let dir = std::env::temp_dir().join("easyofd_ffi_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ffi_test.ofd");
        let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();

        // 创建 writer 并写入
        // SAFETY: 测试中所有指针均为有效分配。
        unsafe {
            let writer = easyofd_writer_new();
            assert!(!writer.is_null());

            let text = CString::new("FFI 测试文本").unwrap();
            let rc = easyofd_writer_add_text_page(writer, 210.0, 297.0, 20.0, 30.0, text.as_ptr());
            assert_eq!(rc, 0);

            let rc = easyofd_writer_build_to_file(writer, path_c.as_ptr());
            assert_eq!(rc, 0);

            easyofd_writer_free(writer);
        }

        // 从文件读取
        // SAFETY: path_c 仍有效。
        unsafe {
            let reader = easyofd_reader_from_file(path_c.as_ptr());
            assert!(!reader.is_null());

            let count = easyofd_page_count(reader);
            assert_eq!(count, 1);

            let merged = easyofd_page_text_merged(reader, 1);
            assert!(!merged.is_null());
            // SAFETY: merged 是有效的 CString 指针。
            let text = CStr::from_ptr(merged).to_str().unwrap();
            assert!(text.contains("FFI 测试文本"));
            easyofd_string_free(merged);

            easyofd_reader_free(reader);
        }

        let _ = std::fs::remove_file(&path);
    }

    /// 测试 roundtrip：从字节创建 reader。
    #[test]
    fn test_roundtrip_bytes() {
        // 先用 writer 生成字节
        // SAFETY: 测试中所有指针均为有效分配。
        let bytes = unsafe {
            let writer = easyofd_writer_new();
            assert!(!writer.is_null());

            let text = CString::new("字节测试").unwrap();
            easyofd_writer_add_text_page(writer, 210.0, 297.0, 10.0, 20.0, text.as_ptr());
            let mut out_len: usize = 0;
            let data_ptr = easyofd_writer_build_to_bytes(writer, std::ptr::addr_of_mut!(out_len));
            assert!(!data_ptr.is_null());
            assert!(out_len > 0);

            let data = std::slice::from_raw_parts(data_ptr, out_len).to_vec();
            easyofd_bytes_free(data_ptr, out_len);
            easyofd_writer_free(writer);
            data
        };

        // 从字节读取
        // SAFETY: bytes 有效。
        unsafe {
            let reader = easyofd_reader_from_bytes(bytes.as_ptr(), bytes.len());
            assert!(!reader.is_null());

            let count = easyofd_page_count(reader);
            assert_eq!(count, 1);

            let all_text = easyofd_extract_all_text(reader);
            assert!(!all_text.is_null());
            // SAFETY: all_text 是有效的 CString 指针。
            let text = CStr::from_ptr(all_text).to_str().unwrap();
            assert!(text.contains("字节测试"));
            easyofd_string_free(all_text);

            easyofd_reader_free(reader);
        }
    }

    /// 测试错误处理：无效路径。
    #[test]
    fn test_error_invalid_path() {
        // SAFETY: 测试中指针有效。
        unsafe {
            let bad_path = CString::new("/nonexistent/path/file.ofd").unwrap();
            let reader = easyofd_reader_from_file(bad_path.as_ptr());
            assert!(reader.is_null());

            let err = easyofd_last_error();
            assert!(!err.is_null());
            // SAFETY: err 是有效的 CString 指针。
            let msg = CStr::from_ptr(err).to_str().unwrap();
            assert!(!msg.is_empty());
            easyofd_string_free(err);
        }
    }

    /// 测试错误处理：无效数据。
    #[test]
    fn test_error_invalid_bytes() {
        // SAFETY: 测试中指针有效。
        unsafe {
            let bad_data = b"not a zip file";
            let reader = easyofd_reader_from_bytes(bad_data.as_ptr(), bad_data.len());
            assert!(reader.is_null());

            let err = easyofd_last_error();
            assert!(!err.is_null());
            easyofd_string_free(err);
        }
    }

    /// 测试错误处理：越界页码。
    #[test]
    fn test_error_out_of_range_page() {
        // SAFETY: 测试中所有指针均为有效分配。
        unsafe {
            let writer = easyofd_writer_new();
            let text = CString::new("page1").unwrap();
            easyofd_writer_add_text_page(writer, 210.0, 297.0, 0.0, 0.0, text.as_ptr());
            let mut out_len: usize = 0;
            let data_ptr = easyofd_writer_build_to_bytes(writer, std::ptr::addr_of_mut!(out_len));
            let bytes = std::slice::from_raw_parts(data_ptr, out_len).to_vec();
            easyofd_bytes_free(data_ptr, out_len);
            easyofd_writer_free(writer);

            let reader = easyofd_reader_from_bytes(bytes.as_ptr(), bytes.len());
            assert!(!reader.is_null());

            // 页码 0 无效
            let rc = easyofd_page_texts(reader, 0, ptr::null_mut(), 0);
            assert!(rc < 0);
            let err = easyofd_last_error();
            assert!(!err.is_null());
            easyofd_string_free(err);

            // 页码 2 越界
            let rc = easyofd_page_texts(reader, 2, ptr::null_mut(), 0);
            assert!(rc < 0);
            let err = easyofd_last_error();
            assert!(!err.is_null());
            easyofd_string_free(err);

            easyofd_reader_free(reader);
        }
    }

    /// 测试 NULL 安全：free(NULL) 不 panic。
    #[test]
    fn test_null_free_safety() {
        // SAFETY: 测试 NULL 安全性——所有 free 函数应接受 NULL。
        unsafe {
            easyofd_reader_free(ptr::null_mut());
            easyofd_writer_free(ptr::null_mut());
            easyofd_string_free(ptr::null_mut());
            easyofd_bytes_free(ptr::null_mut(), 0);
        }
    }

    /// 测试多页文档。
    #[test]
    fn test_multi_page() {
        // SAFETY: 测试中所有指针均为有效分配。
        unsafe {
            let writer = easyofd_writer_new();

            let t1 = CString::new("第一页").unwrap();
            let t2 = CString::new("第二页").unwrap();
            let t3 = CString::new("第三页").unwrap();
            easyofd_writer_add_text_page(writer, 210.0, 297.0, 10.0, 20.0, t1.as_ptr());
            easyofd_writer_add_text_page(writer, 210.0, 297.0, 10.0, 20.0, t2.as_ptr());
            easyofd_writer_add_text_page(writer, 210.0, 297.0, 10.0, 20.0, t3.as_ptr());

            let mut out_len: usize = 0;
            let data_ptr = easyofd_writer_build_to_bytes(writer, std::ptr::addr_of_mut!(out_len));
            let bytes = std::slice::from_raw_parts(data_ptr, out_len).to_vec();
            easyofd_bytes_free(data_ptr, out_len);
            easyofd_writer_free(writer);

            let reader = easyofd_reader_from_bytes(bytes.as_ptr(), bytes.len());
            assert!(!reader.is_null());
            assert_eq!(easyofd_page_count(reader), 3);

            // 提取每页文本
            let mut buf: [*mut c_char; 10] = [ptr::null_mut(); 10];
            let n = easyofd_page_texts(reader, 2, buf.as_mut_ptr(), 10);
            assert!(n >= 1);
            // SAFETY: buf[0] 是有效的 CString 指针。
            let text = CStr::from_ptr(buf[0]).to_str().unwrap();
            assert!(text.contains("第二页"));
            for i in 0..n as usize {
                easyofd_string_free(buf[i]);
            }

            // 全文提取
            let all = easyofd_extract_all_text(reader);
            assert!(!all.is_null());
            // SAFETY: all 是有效的 CString 指针。
            let all_str = CStr::from_ptr(all).to_str().unwrap();
            assert!(all_str.contains("第一页"));
            assert!(all_str.contains("第三页"));
            easyofd_string_free(all);

            easyofd_reader_free(reader);
        }
    }
}
