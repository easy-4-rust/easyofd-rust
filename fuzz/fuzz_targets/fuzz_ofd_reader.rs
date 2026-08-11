//! 模糊测试目标：easyofd_reader::OfdReader::from_bytes
//!
//! 覆盖函数：`OfdReader::from_bytes(data: &[u8]) -> OfdResult<Self>`
//! 目标：任意字节当 OFD 文件解析不 panic/不崩溃。
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // OfdReader::from_bytes 返回 Result，错误一律忽略
    let _ = easyofd_reader::OfdReader::from_bytes(data);
});
