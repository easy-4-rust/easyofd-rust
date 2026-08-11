# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
