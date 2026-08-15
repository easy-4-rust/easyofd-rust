/**
 * easyofd_ffi.h — C ABI bindings for easyofd OFD document operations.
 *
 * 所有权约定：
 *   - easyofd_reader / easyofd_writer 由 create 函数返回，调用方必须用对应 free 函数释放。
 *   - easyofd_last_error() 返回的字符串由 Rust 分配，调用方必须用 easyofd_string_free() 释放。
 *   - easyofd_page_texts() 写入 out 数组的每个字符串由 Rust 分配，调用方必须逐一用
 *     easyofd_string_free() 释放（或在一次性使用后调用 easyofd_page_texts_free()）。
 *
 * 错误处理约定：
 *   - 返回指针的函数在错误时返回 NULL，调用方可通过 easyofd_last_error() 获取错误信息。
 *   - 返回 int32_t 的函数在错误时返回负数，调用方可通过 easyofd_last_error() 获取错误信息。
 *   - easyofd_last_error() 为线程局部存储，每次 FFI 调用失败后自动设置。
 *
 * 线程安全：
 *   - 每个 easyofd_reader / easyofd_writer 实例非线程安全，请勿并发使用同一实例。
 *   - easyofd_last_error() 使用线程局部存储，各线程独立。
 */

#ifndef EASYOFD_FFI_H
#define EASYOFD_FFI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * 不透明句柄：OFD 文档读取器。
 */
typedef struct easyofd_reader easyofd_reader;

/**
 * 不透明句柄：OFD 文档写入器。
 */
typedef struct easyofd_writer easyofd_writer;

/**
 * 从文件路径创建读取器。
 *
 * @param path 以 null 结尾的 UTF-8 文件路径。
 * @return 读取器句柄，失败返回 NULL。错误信息通过 easyofd_last_error() 获取。
 */
easyofd_reader* easyofd_reader_from_file(const char* path);

/**
 * 从内存字节创建读取器。
 *
 * @param data OFD 文件内容指针（调用方持有所有权）。
 * @param len  数据字节数。
 * @return 读取器句柄，失败返回 NULL。错误信息通过 easyofd_last_error() 获取。
 */
easyofd_reader* easyofd_reader_from_bytes(const uint8_t* data, size_t len);

/**
 * 释放读取器句柄。传入 NULL 安全（无操作）。
 */
void easyofd_reader_free(easyofd_reader* reader);

/**
 * 获取文档页数。
 *
 * @param reader 读取器句柄（不得为 NULL）。
 * @return 页数（>= 0）。
 */
int32_t easyofd_page_count(const easyofd_reader* reader);

/**
 * 提取指定页面的文本内容。
 *
 * @param reader 读取器句柄（不得为 NULL）。
 * @param page   页码（从 1 开始）。
 * @param out    输出字符串数组（调用方分配，每个元素由 Rust CString 分配）。
 * @param cap    out 数组容量。
 * @return 实际写入的文本条数；若 page 越界或错误返回负数。
 *         错误信息通过 easyofd_last_error() 获取。
 *
 * 注意：out 中每个字符串由 Rust 分配，调用方须用 easyofd_string_free() 逐一释放。
 */
int32_t easyofd_page_texts(const easyofd_reader* reader,
                           int32_t page,
                           char** out,
                           int32_t cap);

/**
 * 提取指定页面的全部文本合并为一个字符串。
 *
 * @param reader 读取器句柄（不得为 NULL）。
 * @param page   页码（从 1 开始）。
 * @return 合并后的文本字符串（Rust CString 分配），失败返回 NULL。
 *         调用方须用 easyofd_string_free() 释放。
 */
char* easyofd_page_text_merged(const easyofd_reader* reader, int32_t page);

/**
 * 提取全部页面的文本（以 "---" 分隔）。
 *
 * @param reader 读取器句柄（不得为 NULL）。
 * @return 全文字符串（Rust CString 分配），失败返回 NULL。
 *         调用方须用 easyofd_string_free() 释放。
 */
char* easyofd_extract_all_text(const easyofd_reader* reader);

/**
 * 创建空的写入器。
 *
 * @return 写入器句柄，失败返回 NULL。
 */
easyofd_writer* easyofd_writer_new(void);

/**
 * 向写入器添加文本页。
 *
 * @param writer  写入器句柄（不得为 NULL）。
 * @param width   页面宽度（mm）。
 * @param height  页面高度（mm）。
 * @param text_x  文本 X 坐标（mm）。
 * @param text_y  文本 Y 坐标（mm）。
 * @param text    以 null 结尾的 UTF-8 文本内容（调用方持有所有权）。
 * @return 0 成功，负数失败。错误信息通过 easyofd_last_error() 获取。
 */
int32_t easyofd_writer_add_text_page(easyofd_writer* writer,
                                     double width, double height,
                                     double text_x, double text_y,
                                     const char* text);

/**
 * 将写入器内容构建为 OFD 文件并写入磁盘。
 *
 * @param writer 写入器句柄（不得为 NULL）。
 * @param path   输出文件路径（null 结尾 UTF-8）。
 * @return 0 成功，负数失败。错误信息通过 easyofd_last_error() 获取。
 */
int32_t easyofd_writer_build_to_file(easyofd_writer* writer, const char* path);

/**
 * 将写入器内容构建为 OFD 字节并返回。
 *
 * @param writer  写入器句柄（不得为 NULL）。
 * @param out_len 输出字节数。
 * @return 指向 OFD 字节的指针（Rust 分配），失败返回 NULL。
 *         调用方须用 easyofd_bytes_free() 释放。
 */
uint8_t* easyofd_writer_build_to_bytes(easyofd_writer* writer, size_t* out_len);

/**
 * 释放写入器句柄。传入 NULL 安全（无操作）。
 */
void easyofd_writer_free(easyofd_writer* writer);

/**
 * 获取最近一次 FFI 调用的错误信息。
 *
 * @return 线程局部的错误字符串（Rust CString 分配），无错误时返回 NULL。
 *         调用方须用 easyofd_string_free() 释放。
 */
char* easyofd_last_error(void);

/**
 * 释放 Rust 分配的字符串。传入 NULL 安全（无操作）。
 */
void easyofd_string_free(char* s);

/**
 * 释放 Rust 分配的字节数组。传入 NULL 安全（无操作）。
 */
void easyofd_bytes_free(uint8_t* data, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* EASYOFD_FFI_H */
