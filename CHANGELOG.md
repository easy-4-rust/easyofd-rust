# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-08-13

### 生产就绪与字节级保真
- **真·字节级 roundtrip**：全文规范化比对（元素/属性/文本三维），70 样本
  （含 ofdrw 全部 66 个测试文件）0 ZIP + 0 XML + 0 text 偏差
- raw XML 直通机制（OFD.xml/Document.xml 保真回放，对齐 ofdrw flush 哈希门控语义）
- 元数据逐字段端到端验证（含 Subject 补齐、日期 raw 保真）
- cargo-fuzz 4 target + 每日 CI；archive 规则覆盖 95%+；sign 链路 Result 化
- Cargo.lock 入库（可复现构建）；unsafe 门禁 workspace 编译期强制

### 功能补齐（ofdrw 对齐）
- 验证链 checkSealMatch（印章匹配，GB/T 38540 三步验证完整）
- 关键字跨 TextCode 边界定位 + CTM 仿射变换
- 合并：真实 merge（原 STUB 修复）+ 注解/模板/DrawParam/ColorSpace/字体
  全资源迁移（SM3 去重 + ID 改写）+ 流式化（O(常量)内存/源）
- PKCS12：PBES2(AES/SM4) + 传统 PBE（RFC 7292 KDF + 3DES）
- SM2 自签证书 X.509 扩展；GBT35275 真实 IssuerAndSerialNumber
- PDF 保真：坐标基线修正 + FontCache 字体族映射 + 颜色/粗斜体

### 新 crate（21 crate）
- `easyofd-wasm`：浏览器端 OFD 读取（wasm32 实测编译）
- `easyofd-ffi`：C ABI 绑定（15 函数 + cdylib）
- `easyofd-async`：spawn_blocking 异步门面

### 易用性
- 22 个可运行示例 + usage-guide 索引
- 性能基线：vs ofdrw 7~59x（docs/benchmark-report.md，18 场景）
- 2847 tests / 覆盖率 93%+ / clippy 0 / 6 CI workflows

## [1.0.0] - 2026-08-11

First stable release. 18-crate Rust workspace covering the full OFD (GB/T 33190)
document lifecycle: read, write, layout, convert (PDF/Markdown), sign (SM2/SM3),
verify, encrypt, and CLI tooling. 735+ tests, clippy-clean, `#![forbid(unsafe_code)]`.

### Highlights (P0 -- P8)

- **P0 Baseline**: core read/write, GB/T 38540 SM2WithSM3 signatures, OFD-to-Markdown/PDF, anti-zip-bomb, 5 real-world fixtures
- **P1 Layout**: Div box-model, XY-cut reading-order, `LayoutAnalyzer`
- **P2 Types**: 14 action types, 6 annotation types, attachments, version management
- **P3 Signatures**: SES V1-V5, riding stamp, append mode, CRL/OCSP stubs, timestamp DER
- **P4 Layout 2**: render dispatch, streaming layout, virtual page parser
- **P5 Guard**: V4/V5 signature verification pipeline, `SignatureVerificationResult`
- **P6 Encryption**: SM4 CBC/ECB, archive integrity rules, compliance engine
- **P7 Usability**: `EasyOfd` facade, builder pattern, editor, watermarks, custom tags, 708+ tests
- **P8 Production**: roundtrip comparison framework, baseline conformance, 18-crate publish-ready
- **P8 CLI**: `easyofd-tool` crate with 6 subcommands (info, to-markdown, to-pdf, sign, verify, pages), 6 smoke tests

### Crate Inventory (18)

`easyofd-core`, `easyofd-package`, `easyofd-reader`, `easyofd-writer`, `easyofd-layout`,
`easyofd-markdown`, `easyofd-template`, `easyofd-signature`, `easyofd-convert`,
`easyofd-derive`, `easyofd-derive-impl`, `easyofd-gm`, `easyofd-crypto`, `easyofd-archive`,
`easyofd-graphics2d`, `easyofd-font`, `easyofd-tool`, `easyofd` (facade)

### Performance

- Streaming read: O(1) memory for large files
- Template fill: O(template string length)
- Signature verification: < 10ms per signature (SM2 software implementation)

### Security

- Entire workspace enforces `#![forbid(unsafe_code)]`

## [0.1.0] - 2026-08-10

### Added

- Initial project release with full workspace (12 crates)
- Builder pattern API with `#[derive(OfdModel)]` compile-time reflection
- Stream writer for page-by-page ZIP generation
- SAX-based OFD reader with visitor pattern
- Template fill engine with `{key}` placeholder replacement
- Atomic file output (same-directory temp + rename)
- OFD editor (open, modify, save)
- Layout analysis for deterministic reading-order
- 371+ tests across the workspace
