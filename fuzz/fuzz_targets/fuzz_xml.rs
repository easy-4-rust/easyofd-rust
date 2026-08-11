//! 模糊测试目标：easyofd_core::xml_parse::parse_xml_to_nodes
//!
//! 覆盖函数：`parse_xml_to_nodes(xml: &str) -> Result<XmlNode, String>`
//! 目标：任意 XML 字节 → 转为 &str → 解析不 panic。
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // parse_xml_to_nodes 接受 &str，先做 UTF-8 转换；
    // 非 UTF-8 输入直接丢弃（fuzzer 会生成合法 UTF-8 变体）。
    if let Ok(xml) = std::str::from_utf8(data) {
        // 错误一律忽略，重点检测 panic/abort
        let _ = easyofd_core::xml_parse::parse_xml_to_nodes(xml);
    }
});
