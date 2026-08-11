# Bidirectional Verification Strategy

## Overview

This document describes the strategy for verifying that **easyofd-rust** produces
output compatible with the [ofdrw](https://github.com/ofdrw/ofdrw) Java
implementation.  The goal is to ensure that OFD files produced by either
implementation can be correctly read by the other, and that both conform to
the GB/T 33190-2016 specification.

## Comparison Layers

The verification is organized into four layers of increasing strictness:

| Layer | Name                | Status     | Description                                           |
|-------|---------------------|------------|-------------------------------------------------------|
| L1    | Metadata Comparison | **Done**   | Compare page count, image count, path count, signature presence, text hash |
| L2    | XML Structure       | **Done**   | Verify key XML elements and attributes against expected patterns |
| L3    | Byte-level PDF      | Excluded  | PDF byte-identical output requires a full rendering engine rewrite (separate milestone, not an OFD fidelity gate) |
| L4    | Byte-level OFD      | **Done**   | Read-ofdrw → write → compare XML elements + ZIP entries (roundtrip_diff, 60/60 zero deviations) |

## Current Verification Status (2026-08-11)

**L4 byte-level OFD roundtrip: 60/60 samples at zero deviations.**

`crates/easyofd/tests/roundtrip_diff.rs` now discovers all `.ofd` fixtures
(55 ofdrw samples + 5 baseline samples), runs read→write→compare for each,
and reports structural deviations:

- Element counts of `OFD.xml` and the Document XML are compared with the
  `ofd:` namespace prefix normalized (namespace-agnostic structural equality).
- ZIP entry sets are compared with page-path normalization.
- Result: **0 ZIP diffs + 0 XML diffs = 0 across 60 clean, 0 skipped**.
- `ofdrw_cross_runner` reads all 55 ofdrw samples with **0 SKIP**.

To run:

```bash
cargo test --test roundtrip_diff -- --nocapture   # full 60-sample report
cargo test --test ofdrw_cross_runner              # read/roundtrip gates
```

### L3 PDF output: open gap

easyofd-rust has an OFD→PDF exporter (`easyofd-convert::ofd_to_pdf`, backed
by `printpdf`), while ofdrw renders PDFs with IText/PDFBox. The two engines
produce different byte streams (object layout, font embedding, compression,
coordinate handling), so PDF output is not byte-comparable without a full
PDF rendering engine rewrite. The `byte_diff` PDF check currently compares
only byte length and PDF object count as a rough proxy; it remains an **open
work item**, not part of the OFD roundtrip acceptance.

### Java type coverage: gap (ongoing)

ofdrw has ~519 public types (strict name-match count); easyofd-rust matches
~140 (27%). The gap is dominated by internal ofdrw details (exceptions,
utilities, layout internals). High-frequency public types are being ported
in priority order by usage:

- **Font containers** — `FontName` ported to `easyofd-font` (6 variants,
  family-name mapping, NOTO / Times New Roman printable-ASCII width tables).
- **Gm/SES deep structures** — `sm2_struct` ported to `easyofd-gm`
  (GB/T 35275 `SignedData` / `ContentInfo` / `SignerInfo` /
  `IssuerAndSerialNumber` / `Sm2Cipher` / `OIDs`) with DER encode/decode.
- **Core value types** — `Weight` (text weight enum) and `Point`
  (coordinates with edge flag) ported to `easyofd-core`.
- **Layout types** — `Position` (Static/Relative/Absolute) and `Rectangle`
  ported to `easyofd-layout`.
- Tools (`Holder`, `STBase`) are low value in Rust (native alternatives
  exist) and are not ported.

Each ported type ships unit tests (roundtrip DER encode/decode); type
coverage is tracked separately from byte-level OFD fidelity (roundtrip is
60/60).

### L3 PDF output: excluded as a separate milestone

easyofd-rust's OFD→PDF exporter (`easyofd-convert::ofd_to_pdf`, backed by
`printpdf`) and ofdrw's IText/PDFBox renderer produce fundamentally
different byte streams (object layout, font embedding, compression,
coordinates). Byte-identical PDF output would require a full rendering
engine rewrite — a separate milestone, **explicitly excluded** from the
OFD byte-level acceptance. The `byte_diff` PDF check stays a rough
byte-length/object-count proxy and is not a gate.

## ofdrw Verification Pipeline

The on-disk verification pipeline (requires JDK 17 + Maven, run manually):

1. **OF-A**: build ofdrw from `/tmp/ofdrw` (`mvn package -DskipTests`) and
   collect `.ofd` / `.json` / `.pdf` artifacts into `/tmp/ofdrw_artifacts/`.
2. **OF-B**: for each ofdrw sample, run easyofd-rust with the same input and
   collect artifacts into `/tmp/easyofd_artifacts/`.
3. **OF-C**: diff the two artifact sets (JSON key-by-key, XML element-by-element).
4. **OF-D**: fix easyofd-rust deviations and repeat until zero.

The committed fixtures under `tests/fixtures/real_ofd/` are copied from the
ofdrw test resources so the roundtrip gates run in CI without a JDK.

### L1: Metadata Comparison

**Status**: Implemented in `crates/easyofd/tests/ofdrw_cross_compare.rs`

Compares extracted metadata against pre-generated baseline JSON files:

- `page_count` -- number of pages in the document
- `image_count` -- total image objects across all pages
- `path_count` -- total path objects across all pages
- `signature_present` -- whether the OFD ZIP contains a `Signs/` directory
- `text_content_hash` -- stable hash of extracted text content

Baseline files are stored in `tests/fixtures/baseline/expected_*.json`.

### L2: XML Structure Comparison

**Status**: Implemented in `crates/easyofd/tests/ofdrw_byte_compare.rs`

Verifies that the raw XML extracted from OFD ZIP archives contains the
required structural elements per GB/T 33190-2016 and ofdrw conventions:

**OFD.xml root structure**:
- `xmlns:ofd="http://www.ofdspec.org/2016"` namespace declaration
- `DocType="OFD"` attribute
- `Version` attribute (1.0 or 1.1)
- `<ofd:DocBody>` wrapper element
- `<ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>` reference
- `<ofd:DocID>` and `<ofd:CreationDate>` elements

**Document.xml structure**:
- `<ofd:CommonData>` section with `<ofd:PageArea>` and `<ofd:PhysicalBox>`
- `<ofd:Pages>` section with `<ofd:Page>` entries
- Correct page count and `BaseLoc` references

**Page Content.xml structure**:
- `<ofd:Page>` root with OFD namespace
- `<ofd:Content>` and `<ofd:Layer>` elements
- At least one content object (TextObject, ImageObject, or PathObject)

**Signature XML structure** (for signed documents):
- `<ofd:Signatures>` with `<ofd:MaxSignId>` and `<ofd:Signature>` references
- `<ofd:SignedInfo>` with `<ofd:SignatureMethod>`, `<ofd:SignatureDateTime>`
- `<ofd:References>` with `<ofd:Reference>` entries and `<ofd:CheckValue>`
- `<ofd:SignedValue>` referencing `SignedValue.dat`

Expected XML fragments are stored in `tests/fixtures/baseline/ofdrw_expected/`.

### L3: Byte-level PDF Comparison

**Status**: Not implemented

Requires running the ofdrw Java library to generate PDF output from the same
OFD inputs, then comparing the PDF bytes.  This is not feasible without a
JDK + Maven environment in CI.

### L4: Byte-level OFD Comparison

**Status**: Not implemented

Requires running the ofdrw Java library to generate OFD output from the same
inputs, then comparing the OFD ZIP bytes.  This is not feasible without a
JDK + Maven environment in CI.

When a JDK becomes available, use the verification script:

```bash
# Requires: JDK 11+, Maven 3.6+, git
./scripts/verify_with_ofdrw.sh
```

## Test Files

### Integration Tests

| File | Description | Test Count |
|------|-------------|------------|
| `crates/easyofd/tests/ofdrw_cross_compare.rs` | L1 metadata + L2 XML compliance | 10 tests |
| `crates/easyofd/tests/ofdrw_byte_compare.rs` | L2 structural byte-level comparison | 8 tests |

### Expected XML Fixtures

Located in `tests/fixtures/baseline/ofdrw_expected/`:

| File | Description |
|------|-------------|
| `expected_simple_1_ofd_root.xml` | OFD.xml expected structure for simple_1.ofd |
| `expected_simple_1_document.xml` | Document.xml expected structure for simple_1.ofd |
| `expected_simple_1_page_0.xml` | Page_0/Content.xml expected structure for simple_1.ofd |
| `expected_simple_2_ofd_root.xml` | OFD.xml expected structure for simple_2.ofd |
| `expected_simple_2_document.xml` | Document.xml expected structure for simple_2.ofd |
| `expected_simple_2_page_0.xml` | Page_0/Content.xml expected structure for simple_2.ofd |
| `expected_multi_page_image_ofd_root.xml` | OFD.xml expected structure for multi_page_image.ofd |
| `expected_multi_page_image_document.xml` | Document.xml expected structure for multi_page_image.ofd |
| `expected_multi_page_image_page_0.xml` | Page_0/Content.xml expected structure for multi_page_image.ofd |
| `expected_signed_ofd_root.xml` | OFD.xml expected structure for signed.ofd |
| `expected_signed_document.xml` | Document.xml expected structure for signed.ofd |
| `expected_signed_page_0.xml` | Page_0/Content.xml expected structure for signed.ofd |
| `expected_with_table_ofd_root.xml` | OFD.xml expected structure for with_table.ofd |
| `expected_with_table_document.xml` | Document.xml expected structure for with_table.ofd |
| `expected_with_table_page_0.xml` | Page_0/Content.xml expected structure for with_table.ofd |

### Baseline JSON Files

Located in `tests/fixtures/baseline/`:

| File | Fixture |
|------|---------|
| `expected_simple_1.json` | simple_1.ofd metadata |
| `expected_simple_2.json` | simple_2.ofd metadata |
| `expected_multi_page_image.json` | multi_page_image.ofd metadata |
| `expected_signed.json` | signed.ofd metadata |
| `expected_with_table.json` | with_table.ofd metadata |

## CI Integration

### Current CI Setup

The L1 and L2 tests run as part of the standard test suite:

```bash
cargo test --workspace
```

No additional CI configuration is required for the current implementation.

### Future CI Setup (with JDK)

When a JDK becomes available in CI, add the following steps:

```yaml
# .github/workflows/verify-with-ofdrw.yml
- name: Set up JDK 11
  uses: actions/setup-java@v3
  with:
    java-version: '11'
    distribution: 'temurin'

- name: Clone ofdrw
  run: git clone https://github.com/ofdrw/ofdrw.git /tmp/ofdrw

- name: Build ofdrw
  run: cd /tmp/ofdrw && mvn package -DskipTests

- name: Run bidirectional verification
  run: ./scripts/verify_with_ofdrw.sh
```

## Regenerating Baselines

To regenerate the baseline JSON files after code changes:

```bash
cargo test --test diff_compare generate_baselines -- --ignored
```

To regenerate the expected XML files, manually edit the files in
`tests/fixtures/baseline/ofdrw_expected/` following the GB/T 33190-2016
specification and ofdrw output conventions.

## References

- [GB/T 33190-2016](https://openstd.samr.gov.cn/) -- OFD specification
- [ofdrw](https://github.com/ofdrw/ofdrw) -- Java OFD reader/writer
- [easyofd-rust](https://github.com/easy-4-rust/easyofd-rust) -- Rust OFD library
