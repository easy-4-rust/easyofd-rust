# easyofd-rust Usage Guide 使用指南

> Step-by-step examples for all easyofd-rust operations.
>
> **Capability notice:** Signature examples demonstrate the complete GB/T 38540 SM2WithSM3 digital signing flow (SES V1/V4/V5, PKCS#12). Font embedding, PDF conversion, SM4 encryption, and document merge are all production-ready as of v0.1.1.

---

## Installation 安装

```toml
[dependencies]
easyofd = "0.1"
```

---

## 1. Creating OFD Documents 创建 OFD 文档

### 1.1 Derive Macro Approach 派生宏方式

The fastest way: annotate a struct with `#[derive(OfdModel)]` and `#[ofd(...)]` attributes.

```rust
use easyofd::{EasyOfd, OfdModel};

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]
struct Certificate {
    #[ofd(x = 30.0, y = 20.0, font = "SimHei", size = 24.0, bold)]
    title: String,

    #[ofd(x = 30.0, y = 50.0, font = "SimSun", size = 14.0)]
    recipient: String,

    #[ofd(x = 30.0, y = 70.0, size = 14.0)]
    date: String,

    #[ofd(x = 150.0, y = 20.0, kind = "image", img_width = 40.0, img_height = 40.0)]
    seal: Vec<u8>,

    #[ofd(ignore)]
    internal_tracking_id: u64,
}

fn generate_certificate() -> easyofd::OfdResult<()> {
    let seals = std::fs::read("seal.png")?;

    let data = vec![
        Certificate {
            title: "Certificate of Completion".into(),
            recipient: "Alice Wang".into(),
            date: "2026-07-21".into(),
            seal: seals.clone(),
            internal_tracking_id: 1,
        },
        Certificate {
            title: "Certificate of Excellence".into(),
            recipient: "Bob Li".into(),
            date: "2026-07-21".into(),
            seal: seals,
            internal_tracking_id: 2,
        },
    ];

    // Each data item → one page
    EasyOfd::write::<Certificate>("certificates.ofd")
        .metadata_title("Certificates")
        .metadata_author("Training Dept")
        .metadata_creator("easyofd-rust")
        .do_write(&data)?;

    Ok(())
}
```

### 1.2 Manual Page Construction 手动页面构建

For maximum control over page layout:

```rust
use easyofd::{EasyOfd, OfdPage, TextObject, ImageObject, PathObject, page_size};

fn create_invoice() -> easyofd::OfdResult<()> {
    let mut page = OfdPage::new(page_size::A4.0, page_size::A4.1);

    // Title
    page.add_text(
        TextObject::new(20.0, 30.0, "INVOICE")
            .font("SimHei")
            .size(24.0)
            .bold()
    );

    // Separator line
    page.add_path(
        PathObject::hline(20.0, 55.0, 190.0)
            .stroke_color(0x333333)
            .stroke_width(0.5)
    );

    // Table header
    page.add_text(TextObject::new(20.0, 60.0, "Item").bold());
    page.add_text(TextObject::new(100.0, 60.0, "Qty").bold());
    page.add_text(TextObject::new(140.0, 60.0, "Price").bold());

    // Table rows
    page.add_text(TextObject::new(20.0, 75.0, "Widget A"));
    page.add_text(TextObject::new(100.0, 75.0, "10"));
    page.add_text(TextObject::new(140.0, 75.0, "$50.00"));

    page.add_text(TextObject::new(20.0, 88.0, "Widget B"));
    page.add_text(TextObject::new(100.0, 88.0, "5"));
    page.add_text(TextObject::new(140.0, 88.0, "$120.00"));

    // Bottom line
    page.add_path(PathObject::hline(20.0, 100.0, 190.0));

    // Total
    page.add_text(
        TextObject::new(100.0, 105.0, "Total: $1,100.00")
            .bold()
    );

    // Company seal
    page.add_image(ImageObject::jpeg(
        150.0, 200.0, 40.0, 40.0,
        std::fs::read("company_seal.jpg")?,
    ));

    // Footer box
    page.add_path(
        PathObject::rect(10.0, 270.0, 190.0, 20.0)
            .stroke_color(0xCCCCCC)
    );
    page.add_text(TextObject::new(20.0, 275.0, "Thank you for your business!").size(10.0));

    EasyOfd::write_pages("invoice.ofd")
        .metadata_title("Invoice #001")
        .do_write(vec![page])
}
```

### 1.3 Multi-Page Documents 多页文档

```rust
fn create_report() -> easyofd::OfdResult<()> {
    let mut pages = Vec::new();

    // Cover page
    let mut cover = OfdPage::new(210.0, 297.0);
    cover.add_text(TextObject::new(50.0, 130.0, "Annual Report 2025")
        .font("SimHei").size(28.0).bold());
    cover.add_text(TextObject::new(50.0, 160.0, "Confidential").size(14.0));
    pages.push(cover);

    // Content pages
    for i in 1..=5 {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, format!("Section {i}")).size(18.0).bold());
        page.add_text(TextObject::new(20.0, 55.0,
            format!("Content for section {i} goes here...")));
        pages.push(page);
    }

    EasyOfd::write_pages("report.ofd")
        .metadata_title("Annual Report 2025")
        .do_write(pages)
}
```

---

## 2. Reading OFD Documents 读取 OFD 文档

### 2.1 Basic Text Extraction

```rust
use easyofd::EasyOfd;

let reader = EasyOfd::read("document.ofd")?;
println!("Total pages: {}", reader.page_count());

for (i, text) in reader.extract_text().iter().enumerate() {
    println!("--- Page {} ---", i + 1);
    println!("{text}");
}
```

### 2.2 Structured Content Access

```rust
use easyofd::{EasyOfd, ContentObject};

let reader = EasyOfd::read("document.ofd")?;

for page in reader.pages() {
    println!("Page: {} × {} mm", page.width, page.height);
    for obj in &page.content {
        match obj {
            ContentObject::Text(t) => {
                println!("  Text at ({}, {}): {}", t.x, t.y, t.text);
            }
            ContentObject::Image(img) => {
                println!("  Image at ({}, {}): {} × {} mm, format: {:?}",
                    img.x, img.y, img.width, img.height, img.format);
            }
            ContentObject::Path(_) => {
                println!("  Path object");
            }
        }
    }
}
```

### 2.3 Processing from Bytes

```rust
let uploaded_bytes: Vec<u8> = get_from_network()?;
let reader = EasyOfd::read_from_bytes(&uploaded_bytes)?;
let all_text = reader.extract_all_text();
```

---

## 3. Template Filling 模板填充

### 3.1 Creating a Template

First, create a template OFD with `{placeholder}` patterns:

```rust
// When creating the template, use {placeholders} in text:
let mut page = OfdPage::new(210.0, 297.0);
page.add_text(TextObject::new(20.0, 30.0, "Invoice: {title}").size(18.0).bold());
page.add_text(TextObject::new(20.0, 60.0, "Amount: {amount}"));
page.add_text(TextObject::new(20.0, 80.0, "Date: {date}"));
page.add_text(TextObject::new(20.0, 100.0, "{notes}"));

let mut writer = OfdWriter::new();
writer.add_page(page);
writer.build_to_file("template.ofd")?;
```

### 3.2 Filling the Template

```rust
use std::collections::HashMap;

let mut data = HashMap::new();
data.insert("title".into(), "INV-2026-0042".into());
data.insert("amount".into(), "$8,500.00".into());
data.insert("date".into(), "2026-07-21".into());
data.insert("notes".into(), "Payment due within 30 days.".into());

let filler = EasyOfd::fill_template("template.ofd", &data)?;
filler.save("invoice-0042.ofd")?;
```

### 3.3 Handling Missing Keys

Keys not present in the data map are **preserved as-is** in the output:

```rust
let mut data = HashMap::new();
data.insert("name".into(), "Alice".into());
// "address" not provided → "{address}" stays in output
```

### 3.4 Batch Filling

```rust
fn batch_generate_invoices(invoices: &[(String, HashMap<String, String>)]) -> easyofd::OfdResult<()> {
    for (filename, data) in invoices {
        EasyOfd::fill_template("template.ofd", data)?
            .save(filename)?;
    }
    Ok(())
}
```

---

## 4. Electronic Signatures 电子签章

### 4.1 Adding a Visual Seal

```rust
use easyofd::{OfdSignatureBuilder, ElectronicSeal};

let seal = ElectronicSeal {
    image_data: std::fs::read("company_seal.png")?,
    name: "Company Official Seal".into(),
    position: (150.0, 200.0),  // mm from top-left
    page: 0,                     // 0-based page index
};

OfdSignatureBuilder::new("contract.ofd")
    .seal(seal)
    .sign()?
    .save("contract-signed.ofd")?;
```

### 4.2 Multiple Seals

```rust
OfdSignatureBuilder::new("contract.ofd")
    .seal(company_seal)
    .seal(supervisor_seal)
    .seal(auditor_seal)
    .sign()?
    .save("fully-sealed.ofd")?;
```

### 4.3 Specifying Algorithm

```rust
use easyofd::SignatureAlgorithm;

OfdSignatureBuilder::new("document.ofd")
    .seal(my_seal)
    .algorithm(SignatureAlgorithm::Sha256WithRsa)
    .sign()?
    .save("signed.ofd")?;
```

### 4.4 Full Cryptographic Signing (SM2WithSM3)

The `sign()` method performs real SM2WithSM3 signing (GB/T 38540) using an
internal SM2 key pair.  The resulting `SignedOfd` carries the SM3 digest
and SM2 signature value; `verify_signature()` validates the roundtrip.

```rust
// Sign with default SM2WithSM3 algorithm
let signed = OfdSignatureBuilder::new("document.ofd")
    .seal(my_seal)
    .sign()?;    // ← generates Signature.xml + SignedInfo + SignedValue

signed.save("fully-signed.ofd")?;

// Verify
let valid = easyofd::verify_signature("fully-signed.ofd")?;
assert!(valid);
```

For multi-signer scenarios (batch signing), use `.add_signature()`:

```rust
use easyofd_signature::SignatureAlgorithm;

let signed = OfdSignatureBuilder::new("document.ofd")
    .algorithm(SignatureAlgorithm::Sm2WithSm3)
    .add_signature(secret_key, vec![seal1, seal2])
    .sign_multiple()?;

signed.save("multi-signed.ofd")?;
```

---

## 5. Custom Fonts 自定义字体

```rust
use easyofd::{EmbeddedFont, FontFormat};

let mut writer = OfdWriter::new();

// Register a TTF font that can be referenced in TextObject::font()
writer.embed_font(EmbeddedFont {
    name: "MyCustomFont".into(),
    data: std::fs::read("my-font.ttf")?,
    format: FontFormat::TrueType,
});

let mut page = OfdPage::new(210.0, 297.0);
page.add_text(TextObject::new(20.0, 30.0, "Custom font text")
    .font("MyCustomFont")
    .size(16.0));
writer.add_page(page);
writer.build_to_file("custom-font.ofd")?;
```

---

## 6. Common Patterns 常见模式

### 6.1 Write → Read Roundtrip

```rust
// Create
let mut page = OfdPage::new(210.0, 297.0);
page.add_text(TextObject::new(10.0, 20.0, "Roundtrip test"));
let bytes = EasyOfd::write_pages_to_bytes(vec![page])?;

// Read back
let reader = EasyOfd::read_from_bytes(&bytes)?;
assert_eq!(reader.page_count(), 1);
assert!(reader.extract_all_text().contains("Roundtrip test"));
```

### 6.2 Write → Sign → Read Pipeline

```rust
use easyofd::{EasyOfd, OfdPage, TextObject, OfdSignatureBuilder, ElectronicSeal};

// Step 1: Create document
let mut page = OfdPage::new(210.0, 297.0);
page.add_text(TextObject::new(20.0, 30.0, "Contract content"));
EasyOfd::write_pages("draft.ofd")
    .metadata_title("Contract")
    .do_write(vec![page])?;

// Step 2: Apply electronic seal (SM2WithSM3 signing)
let seal = ElectronicSeal {
    image_data: std::fs::read("seal.png")?,
    name: "Company Seal".into(),
    position: (150.0, 200.0),
    page: 0,
};
OfdSignatureBuilder::new("draft.ofd")
    .seal(seal)
    .sign()?
    .save("final.ofd")?;

// Step 3: Verify signature
let valid = easyofd::verify_signature("final.ofd")?;
println!("Signature valid: {valid}");

// Step 4: Read back
let reader = EasyOfd::read("final.ofd")?;
println!("Signed document: {} pages", reader.page_count());
```

### 6.3 Template → Fill → Sign Pipeline

```rust
// Prepare data
let mut data = HashMap::new();
data.insert("contract_number".into(), "CT-2026-0042".into());
data.insert("party_a".into(), "Company A Ltd.".into());
data.insert("party_b".into(), "Company B Inc.".into());

// Fill template
let filler = EasyOfd::fill_template("contract_template.ofd", &data)?;
let filled_bytes = filler.into_bytes();

// Save intermediate
std::fs::write("contract-filled.ofd", &filled_bytes)?;

// Sign
OfdSignatureBuilder::new("contract-filled.ofd")
    .seal(company_seal)
    .sign()?
    .save("contract-executed.ofd")?;
```

---

## 7. Error Handling 错误处理

All operations return `OfdResult<T>`:

```rust
use easyofd::{EasyOfd, OfdError};

fn process_ofd(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match EasyOfd::read(path) {
        Ok(reader) => {
            println!("Success: {} pages", reader.page_count());
            println!("{}", reader.extract_all_text());
            Ok(())
        }
        Err(OfdError::Io(e)) => {
            eprintln!("File error: {e}");
            Err(e.into())
        }
        Err(OfdError::Zip(e)) => {
            eprintln!("Not a valid OFD/ZIP: {e}");
            Err(e.into())
        }
        Err(OfdError::Xml(e)) => {
            eprintln!("XML parse error: {e}");
            Err(e.into())
        }
        Err(e) => {
            eprintln!("Other error: {e}");
            Err(e.into())
        }
    }
}
```

---

## 8. Page Size Reference 页面尺寸参考

| Constant 常量 | Dimensions (mm) | Use |
|:---|:---|:---|
| `page_size::A4` | 210 × 297 | Standard document |
| `page_size::A4_LANDSCAPE` | 297 × 210 | Wide tables |
| `page_size::A3` | 297 × 420 | Large diagrams |
| `page_size::LETTER` | 215.9 × 279.4 | US Letter |
| Custom | any (w, h) | OfdPage::new(w, h) |

---

## 9. Best Practices 最佳实践

| Practice | Why |
|:---|:---|
| Use derive macro for repetitive documents | Automatically maps struct → OFD, zero boilerplate |
| Use manual construction for complex layouts | Full control over every element position |
| Embed seal images as PNG | Smaller size, transparent background support |
| Use `{placeholder}` in template text | Enables batch document generation |
| Validate page dimensions | Origin (0,0) is top-left; elements outside page may be clipped |
| Use `page_size::A4` constants | Avoids magic numbers |
| Always handle `OfdResult` | Proper error propagation for I/O, ZIP, and XML errors |

---

## 10. Example Index 示例索引

所有示例位于 `crates/easyofd/examples/`，可直接运行：

```bash
cargo run -p easyofd --example <name>
```

| 示例 | 说明 | 运行命令 |
|:---|:---|:---|
| `01_hello_ofd` | 最小写入 + 读回 roundtrip | `cargo run -p easyofd --example 01_hello_ofd` |
| `02_read_metadata` | 读取元数据、页数、页面尺寸与内容统计 | `cargo run -p easyofd --example 02_read_metadata` |
| `03_text_image_page` | 图文混排：发票布局（文本+图片+路径） | `cargo run -p easyofd --example 03_text_image_page` |
| `04_stream_writer` | 流式写入 100 页大文档，内存占用恒定 | `cargo run -p easyofd --example 04_stream_writer` |
| `05_template_fill` | 模板占位符 `{key}` 批量填充，生成劳动合同 | `cargo run -p easyofd --example 05_template_fill` |
| `06_to_markdown` | OFD 转 Markdown，含转换报告与流式输出 | `cargo run -p easyofd --example 06_to_markdown` |
| `07_sign_verify` | SM2WithSM3 签名 + 验签 + 篡改检测 roundtrip | `cargo run -p easyofd --example 07_sign_verify` |
| `08_merge_docs` | 多文档合并（读取页面 + 写入新文档） | `cargo run -p easyofd --example 08_merge_docs` |
| `09_archive_check` | OFD-A 归档合规规则检查 + 完整性校验 | `cargo run -p easyofd --example 09_archive_check` |
| `10_convert_pdf` | OFD 与 PDF 双向转换（全页/部分页） | `cargo run -p easyofd --example 10_convert_pdf` |
| `11_keyword_search` | 关键字定位（含跨 TextCode 边界匹配） | `cargo run -p easyofd --example 11_keyword_search` |
| `12_encrypt_decrypt` | SM4-CBC 加解密 roundtrip + 错误密钥测试 | `cargo run -p easyofd --example 12_encrypt_decrypt` |

此外还有已有的专题示例：

| 示例 | 说明 |
|:---|:---|
| `action_uri` | URI 超链接动作（GB/T 33190 第 15 章） |
| `annotation` | 注释功能（文本/高亮/印章/链接/手写） |
| `batch_sign` | 批量签章（多文档 + 多签章模式） |
| `benchmark` | 读取与 Markdown 转换性能基准 |
| `convert_pdf` | OFD/PDF 转换（旧版） |
| `markdown_export` | Markdown 导出（旧版） |
| `read_simple` | 简单读取示例 |
| `read_with_visitor` | 逐页 visitor 模式读取 |
| `signature_roundtrip` | 签名 roundtrip（旧版） |
| `write_simple` | 简单写入示例 |
