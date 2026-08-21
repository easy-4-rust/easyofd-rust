<a id="readme-top"></a>

<div align="center">

# easyofd-rust

**Idiomatic Rust OFD library — Builder pattern, compile-time derive, GB/T 33190-2016 compliant**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)
[![crates.io](https://img.shields.io/crates/v/easyofd.svg)](https://crates.io/crates/easyofd)

[English](./README.md) · [简体中文](./README.zh-CN.md)

[Features](#features) · [Architecture](#architecture) ·
[Quick Start](#quick-start) · [API Reference](#api-reference) ·
[Design Principles](#design-principles) · [Roadmap](#roadmap) ·
[Contributing](#contributing)

</div>

---

> **Current version**: `0.1.1` (published on crates.io)<br>
> **MSRV**: Rust `1.88`<br>
> **Edition**: `2024`<br>
> **Workspace Resolver**: `3`<br>
> **License**: Apache-2.0

`easyofd-rust` is a pure-Rust implementation of the OFD (Open Fixed-layout Document)
national standard **GB/T 33190-2016**, including **GB/T 38540** digital signatures.
It covers the full document lifecycle: **create**, **read**, **stream-write**,
**template fill**, **edit**, **merge**, **encrypt**, **sign & verify**,
**OFD → Markdown** and **OFD ↔ PDF** conversion.

The workspace mirrors the Java [ofdrw](https://github.com/ofdrw/ofdrw) feature set
with byte-level output fidelity (70/70 ofdrw test samples round-trip with zero deviations).
Inspired by [Alibaba EasyExcel](https://github.com/alibaba/easyexcel).

---

## Features

| Feature | Status | Description |
|:---|:---:|:---|
| Create OFD | ✅ | Text, images, paths, metadata; fluent Builder API |
| `#[derive(OfdModel)]` | ✅ | Compile-time reflection; zero runtime cost |
| Stream Writer | ✅ | Page-by-page ZIP writing; constant memory per page |
| Editor | ✅ | Open → modify (add text, pages, watermarks) → save |
| Read OFD | ✅ | XML parsing, page visitor, safe ZIP validation |
| OFD → Markdown | ✅ | Deterministic reading-order, image export, loss report |
| Template fill | ✅ | `{key}` placeholder replacement, binary-preserving |
| Merge | ✅ | Streaming multi-document merge; full resource migration (ColorSpace/Font/DrawParam/templates) with SM3 dedup |
| Digital signatures | ✅ | GB/T 38540 SM2WithSM3; SES V1/V4/V5; seal-match verification (`checkSealMatch`); PKCS#12 (PBES2 AES/SM4 + legacy PBE 3DES) |
| Encryption | ✅ | SM4-CBC/ECB; archive integrity rules & compliance engine |
| PDF ↔ OFD | ✅ | PDF fidelity: coordinate baseline, FontCache family mapping, color/bold/italic |
| Keyword search | ✅ | Cross-`TextCode`-boundary keyword location + CTM affine transforms |
| Vector paths | ✅ | hline, vline, rect with stroke/fill; canvas drawing |
| Custom fonts | ✅ | Font resource generation + FontLoader similar-substitution |

## Byte-level fidelity with ofdrw

Roundtrip verification (`read → write → compare`) against every `.ofd` sample produced
by the Java ofdrw test suite:

```
70/70 samples, 0 ZIP + 0 XML + 0 text deviations, 0 skipped
```

Raw-XML pass-through for `OFD.xml` / `Document.xml` matches ofdrw's flush semantics,
and metadata (Creator / Author / ModDate / DocID / …) is preserved field-by-field.

---

## Architecture

```
easyofd-rust (21 crates)
├── easyofd             🎯 Facade — EasyOfd::write/read/to_markdown/fill_template/…
├── easyofd-core        🧩 Types, traits, errors, data model
├── easyofd-derive      ⚡ Proc-macro shim
├── easyofd-derive-impl ⚙️ All derive logic
├── easyofd-reader      📖 OFD parsing + page visitor
├── easyofd-writer      ✍️ ZIP/XML generation + stream writer + editor
├── easyofd-package     🛡️ ZIP limits, safe paths, atomic replacement
├── easyofd-layout      📐 Deterministic reading-order analysis
├── easyofd-markdown    📝 Streaming OFD → Markdown + loss report
├── easyofd-template    📋 Placeholder replacement engine
├── easyofd-signature   🔐 GB/T 38540 signatures, SES V1-V5, seal verification
├── easyofd-convert     🔄 OFD ↔ PDF conversion
├── easyofd-gm          🇨🇳 SM2/SM3/SM4 integration (GM algorithms)
├── easyofd-crypto      🔒 OFD encryption infrastructure (SM4, PKCS#12)
├── easyofd-archive     🗄️ Archive compliance rules engine
├── easyofd-graphics2d  🎨 2D graphics abstraction (ofdrw-graphics2d)
├── easyofd-font        🔤 Font management & embedding
├── easyofd-tool        🧰 CLI: info / to-markdown / to-pdf / sign / verify / pages / merge
├── easyofd-wasm        🌐 WASM bindings for browser-side reading (wasm32)
├── easyofd-ffi         🅲 C ABI bindings (15 functions, cdylib)
└── easyofd-async       ⚡ Async facade (spawn_blocking bridge)
```

See [docs/easyofd-rust-Architecture.md](docs/easyofd-rust-Architecture.md) for the full architecture document.

---

## Quick Start

```toml
[dependencies]
easyofd = "0.1"
```

### 1. Write with Derive Macro

```rust
use easyofd::{EasyOfd, OfdModel};

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]
struct Invoice {
    #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
    title: String,
    #[ofd(x = 20.0, y = 50.0)]
    amount: String,
}

let data = vec![
    Invoice { title: "Invoice #001".into(), amount: "$100.00".into() },
    Invoice { title: "Invoice #002".into(), amount: "$200.00".into() },
];

EasyOfd::write::<Invoice>("invoices.ofd")
    .metadata_title("Monthly Invoices")
    .do_write(&data)?;
```

### 2. Manual Page Construction

```rust
use easyofd::{EasyOfd, TextObject, ImageObject, OfdPage};

let mut page = OfdPage::new(210.0, 297.0); // A4
page.add_text(TextObject::new(20.0, 30.0, "Hello OFD!").size(24.0).bold());
page.add_text(TextObject::new(20.0, 60.0, "Normal text"));
page.add_image(ImageObject::jpeg(150.0, 30.0, 30.0, 30.0, jpeg_bytes));

EasyOfd::write_pages_to("output.ofd", vec![page])?;
```

### 3. Stream Writer (Large Documents)

```rust
use easyofd::{EasyOfd, OfdPage, TextObject};

let file = std::fs::File::create("large.ofd")?;
let mut writer = EasyOfd::stream_writer(file);
for i in 1..=100_000 {
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(10.0, 10.0, format!("Page {i}")));
    writer.write_page(page)?;
}
writer.finish()?;
```

### 4. Read OFD

```rust
use easyofd::EasyOfd;

// Visitor pattern — pages are not retained in memory
let visited = EasyOfd::read_pages("input.ofd")
    .page_range(1, 10)
    .do_read(|page_number, page| {
        println!("Page {page_number}: {} objects", page.content.len());
        Ok(())
    })?;
```

### 5. OFD → Markdown

```rust
use easyofd::EasyOfd;

// In-memory
let result = EasyOfd::to_markdown("input.ofd").do_convert()?;
println!("{}", result.markdown);
println!("Pages: {}, Losses: {}", result.report.pages_converted, result.report.losses.len());

// Stream to file
use std::fs::File;
EasyOfd::to_markdown("input.ofd")
    .convert_to(File::create("output.md")?)?;
```

### 6. Template Fill

```rust
use easyofd::EasyOfd;
use std::collections::HashMap;

let mut data = HashMap::new();
data.insert("name".to_string(), "Alice".to_string());
data.insert("amount".to_string(), "$1,234.00".to_string());

EasyOfd::fill_template("template.ofd", &data)?.save("filled.ofd")?;
```

### 7. Edit Existing OFD

```rust
use easyofd::{OfdEditor, TextObject, Watermark};

let mut editor = OfdEditor::open("input.ofd")?;
editor.add_text_to_page(0, TextObject::new(10.0, 40.0, "Added text"))?;
editor.apply_watermarks(&[Watermark::text("CONFIDENTIAL").position(50.0, 150.0)]);
editor.save("edited.ofd")?;
```

### 8. Sign (GB/T 38540)

```rust
use easyofd_signature::{ElectronicSeal, OfdSignatureBuilder, SignatureAlgorithm};

let signed = OfdSignatureBuilder::new("input.ofd")
    .algorithm(SignatureAlgorithm::SM2WithSM3)
    .seal(seal_info) // SealInfo / ElectronicSeal — see examples/07_sign_verify.rs
    .add_signature(secret_key, vec![seal])
    .sign()?; // → SignedOfd
signed.save("signed.ofd")?;
```

> Verification (SES V1/V4/V5 containers, `check_seal_match`, chain/CRL/OCSP) and
> tamper-detection are exercised in `crates/easyofd-signature/tests/` and
> `examples/07_sign_verify.rs`.

> More complete examples — digital seal (SES), batch signing, merge, archive check,
> encryption, keyword search — live in `crates/easyofd/examples/` (22 runnable examples,
> indexed in [docs/usage-guide.md](docs/usage-guide.md)).

---

## API Reference

### Entry Point

All operations go through `EasyOfd` static methods:

| Method | Returns | Purpose |
|---|---|---|
| `EasyOfd::write::<T>(path)` | `OfdWriterBuilder<T>` | Typed write with `OfdModel` |
| `EasyOfd::write_pages(path)` | `PageWriterBuilder` | Manual page write |
| `EasyOfd::write_pages_to(path, pages)` | `OfdResult<()>` | One-shot file write |
| `EasyOfd::write_pages_to_bytes(pages)` | `OfdResult<Vec<u8>>` | One-shot bytes |
| `EasyOfd::stream_writer(output)` | `OfdStreamWriter<W>` | Streaming write |
| `EasyOfd::read(path)` | `OfdResult<OfdReader>` | Full read (metadata + pages) |
| `EasyOfd::read_from_bytes(data)` | `OfdResult<OfdReader>` | Read from in-memory bytes |
| `EasyOfd::read_pages(path)` | `OfdReadBuilder` | Page visitor |
| `EasyOfd::to_markdown(path)` | `MarkdownConversionBuilder` | Markdown conversion |
| `EasyOfd::fill_template(path, data)` | `OfdResult<OfdTemplateFiller>` | Template fill |

Signing, verification, encryption, merge and keyword search are provided by the
dedicated crates (`easyofd-signature`, `easyofd-crypto`, `easyofd-tool`) and re-exported
where applicable — see [docs/usage-guide.md](docs/usage-guide.md).

### Core Types

| Type | Purpose |
|---|---|
| `OfdPage` | A page with width, height, and content objects |
| `TextObject` | Positioned text with font, size, weight, color |
| `ImageObject` | Positioned image (JPEG/PNG/BMP/TIFF) |
| `PathObject` | Vector path (SVG-like `d` attribute) |
| `OfdModel` | Trait for mapping Rust structs to OFD pages |
| `OfdError` | Unified error enum (workspace-wide `OfdResult<T>`) |
| `ConversionReport` | Markdown conversion results + losses + warnings |

---

## Design Principles

| Principle | Implementation |
|---|---|
| **Zero unsafe** | `#![forbid(unsafe_code)]` across the workspace (FFI crate exempted with documented `SAFETY` comments) |
| **Fluent Builders** | `mut self → Self` with `#[must_use]` |
| **Compile-time reflection** | `#[derive(OfdModel)]` — zero runtime cost |
| **Single facade** | `EasyOfd` — discoverable static factory |
| **GB/T 33190-2016** | Valid OFD ZIP with correct XML namespaces |
| **Byte-level fidelity** | Raw-XML pass-through + normalized full-text roundtrip vs ofdrw (0 deviations) |
| **Streaming first** | Writer/Reader/Markdown all support page-by-page processing; merge is O(1) memory per source |
| **Separation of concerns** | Each crate has one job; facade wires them together |

---

## Workspace

21 crates across 4 groups (all published to crates.io as `0.1.1`):

| Group | Crates |
|---|---|
| Facade & core | `easyofd`, `easyofd-core`, `easyofd-derive`, `easyofd-derive-impl` |
| Read/write pipeline | `easyofd-reader`, `easyofd-writer`, `easyofd-package`, `easyofd-layout`, `easyofd-markdown`, `easyofd-template` |
| ofdrw-aligned modules | `easyofd-signature`, `easyofd-convert`, `easyofd-gm`, `easyofd-crypto`, `easyofd-archive`, `easyofd-graphics2d`, `easyofd-font` |
| Platform & tooling | `easyofd-tool` (CLI), `easyofd-wasm`, `easyofd-ffi`, `easyofd-async` |

**2860 tests · 22 examples · clippy `-D warnings` clean · coverage 93%+ · 6 CI workflows (3-OS × 2 toolchains)**

---

## Performance vs ofdrw (Java)

Full benchmark methodology and per-scenario results:
[docs/benchmark-report.md](docs/benchmark-report.md) (2026-08-16, Apple Silicon vs OpenJDK 21).

Summary: **7–59× faster than ofdrw** across 18 write/read/roundtrip scenarios.
Rust keeps the whole pipeline in memory (no disk unzip, no layout engine overhead);
caveats on methodology are documented in the report.

---

## Examples

| Example | Description | Run |
|---|---|---|
| `01_hello_ofd` | Minimal OFD creation | `cargo run --example 01_hello_ofd` |
| `02_read_metadata` | Read metadata / DocInfo fields | `cargo run --example 02_read_metadata` |
| `03_text_image_page` | Text + image + vector page | `cargo run --example 03_text_image_page` |
| `04_stream_writer` | Page-by-page streaming | `cargo run --example 04_stream_writer` |
| `05_template_fill` | Template placeholder fill | `cargo run --example 05_template_fill` |
| `06_to_markdown` | OFD → Markdown + loss report | `cargo run --example 06_to_markdown` |
| `07_sign_verify` | SM2WithSM3 sign → verify | `cargo run --example 07_sign_verify` |
| `08_merge_docs` | Multi-document merge | `cargo run --example 08_merge_docs` |
| `09_archive_check` | Archive compliance rules | `cargo run --example 09_archive_check` |
| `10_convert_pdf` | OFD ↔ PDF conversion | `cargo run --example 10_convert_pdf` |
| `11_keyword_search` | Cross-boundary keyword search | `cargo run --example 11_keyword_search` |
| `12_encrypt_decrypt` | SM4 encryption roundtrip | `cargo run --example 12_encrypt_decrypt` |
| `batch_sign` | Batch + multi-signer signing | `cargo run --example batch_sign` |
| `benchmark` | Performance benchmark | `cargo run --release --example benchmark -- 10000` |

Full list (22 examples): [docs/usage-guide.md](docs/usage-guide.md).

---

## Testing

```bash
# All tests (2860)
cargo test --workspace

# Clippy
cargo clippy --workspace -- -D warnings

# Compile-fail tests (derive macro error messages)
cargo test -p easyofd-derive-impl

# ofdrw byte-level parity (70 fixtures)
cargo test -p easyofd --test roundtrip_diff --release
```

---

## Roadmap

| Version | Milestone | Status |
|---|---|:---:|
| v0.1.0 | Initial crates.io release (facade + core) | ✅ published 2026-08-10 |
| v0.1.1 | Full 21-crate workspace: signatures, encryption, PDF, merge, WASM/FFI/async; byte-level ofdrw parity | ✅ published 2026-08-21 |

---

## Contributing

1. Fork and clone
2. `cargo test --workspace` — all tests must pass
3. `cargo clippy --workspace -- -D warnings` — no warnings
4. All new code must have `#[test]` coverage
5. No `unsafe` code — `#![forbid(unsafe_code)]` is enforced (FFI crate exempted)

---

## License

Apache-2.0. See [LICENSE](LICENSE).
