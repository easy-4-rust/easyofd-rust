//! 模糊测试目标：lopdf::Document::load_mem
//!
//! 覆盖函数：`lopdf::Document::load_mem(data: &[u8])`
//! 目标：任意 PDF 字节解析不 panic。
//! 注意：RUSTSEC-2026-0187 是已知深度嵌套 DoS，fuzz 若发现 panic 属预期发现。
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // lopdf 返回 Result，错误一律忽略，重点检测 panic/abort
    let _ = lopdf::Document::load_mem(data);
});
