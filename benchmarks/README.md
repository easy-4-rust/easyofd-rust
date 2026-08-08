# easyofd-rust benchmark contract

Run the deterministic text-page scenario with a release build:

```bash
cargo run --release -p easyofd --example benchmark -- 10000
```

The command emits one JSON object containing the scenario, input size, visited page count,
text checksum input, reader duration and Markdown duration. Compare results only when the Rust
toolchain, OS, CPU, page count and source revision are the same.

Performance claims require, at minimum, these separate scenarios:

- text-only pages: 100, 1,000 and 10,000 pages;
- image-heavy pages with a fixed embedded-byte corpus;
- full-document reading versus `read_pages` visitor reading;
- in-memory Markdown versus `convert_to` streaming output;
- correctness gates for page count, text bytes, exported assets and conversion losses.

Historical local numbers are not a stable baseline. Commit reviewed baseline JSON separately
only after the benchmark corpus and machine metadata are pinned.
