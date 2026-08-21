<a id="readme-top"></a>

<div align="center">

# easyofd-rust

**纯 Rust OFD 库 — Builder 模式、编译期派生宏、GB/T 33190-2016 合规**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)
[![crates.io](https://img.shields.io/crates/v/easyofd.svg)](https://crates.io/crates/easyofd)

[English](./README.md) · [简体中文](./README.zh-CN.md)

[功能](#功能) · [架构](#架构) ·
[快速开始](#快速开始) · [API 参考](#api-参考) ·
[设计原则](#设计原则) · [路线图](#路线图) ·
[贡献](#贡献)

</div>

---

> **当前版本**：`0.1.1`（已发布至 crates.io）<br>
> **MSRV**：Rust `1.88`<br>
> **Edition**：`2024`<br>
> **Workspace Resolver**：`3`<br>
> **许可证**：Apache-2.0

`easyofd-rust` 是 OFD（开放版式文档）国家标准 **GB/T 33190-2016** 的纯 Rust 实现，
并覆盖 **GB/T 38540** 电子签章。提供完整的文档生命周期：**创建**、**读取**、
**逐页流式写入**、**模板填充**、**编辑**、**合并**、**加密**、**签章与验签**、
**OFD → Markdown** 与 **OFD ↔ PDF** 转换。

工作区对标 Java [ofdrw](https://github.com/ofdrw/ofdrw) 的功能全集，输出达到
**字节级一致**（ofdrw 全部 70 个测试样本 roundtrip 零偏差）。设计灵感来自
[Alibaba EasyExcel](https://github.com/alibaba/easyexcel)。

---

## 功能

| 功能 | 状态 | 描述 |
|:---|:---:|:---|
| 创建 OFD | ✅ | 文本、图片、路径、元数据；流畅 Builder API |
| `#[derive(OfdModel)]` | ✅ | 编译期反射；零运行时开销 |
| 流式 Writer | ✅ | 逐页写入 ZIP；每页内存恒定 |
| 编辑器 | ✅ | 打开 → 修改（添加文本、页面、水印）→ 保存 |
| 读取 OFD | ✅ | XML 解析、页面 visitor、安全 ZIP 校验 |
| OFD → Markdown | ✅ | 确定性阅读顺序、图片导出、损失报告 |
| 模板填充 | ✅ | `{key}` 占位符替换，二进制保持 |
| 合并 | ✅ | 流式多文档合并；ColorSpace/Font/DrawParam/模板全资源迁移（SM3 去重） |
| 电子签章 | ✅ | GB/T 38540 SM2WithSM3；SES V1/V4/V5；印章匹配验证（`checkSealMatch`）；PKCS#12（PBES2 AES/SM4 + 传统 PBE 3DES） |
| 加密 | ✅ | SM4-CBC/ECB；归档完整性规则与合规引擎 |
| PDF ↔ OFD | ✅ | PDF 保真：坐标基线修正、FontCache 字体族映射、颜色/粗斜体 |
| 关键字搜索 | ✅ | 跨 `TextCode` 边界定位 + CTM 仿射变换 |
| 矢量路径 | ✅ | 水平线、垂直线、矩形，支持描边/填充；画布绘制 |
| 自定义字体 | ✅ | 字体资源生成 + FontLoader 相似字体替换 |

## 与 ofdrw 的字节级一致性

对 Java ofdrw 测试套件产出的每个 `.ofd` 样本做 roundtrip 验证
（读 → 写 → 全文比对）：

```
70/70 样本，0 ZIP + 0 XML + 0 文本偏差，0 跳过
```

`OFD.xml` / `Document.xml` 的 raw XML 直通机制对齐 ofdrw 的 flush 语义；
元数据（Creator / Author / ModDate / DocID / …）逐字段精确保留。

---

## 架构

```
easyofd-rust（21 个 crate）
├── easyofd             🎯 外观层 — EasyOfd::write/read/to_markdown/fill_template/…
├── easyofd-core        🧩 类型、trait、错误、数据模型
├── easyofd-derive      ⚡ proc-macro 薄入口
├── easyofd-derive-impl ⚙️ 全部派生逻辑
├── easyofd-reader      📖 OFD 解析 + 页面 visitor
├── easyofd-writer      ✍️ ZIP/XML 生成 + 流式 Writer + 编辑器
├── easyofd-package     🛡️ ZIP 限制、安全路径、原子替换
├── easyofd-layout      📐 确定性阅读顺序分析
├── easyofd-markdown    📝 流式 OFD → Markdown + 损失报告
├── easyofd-template    📋 占位符替换引擎
├── easyofd-signature   🔐 GB/T 38540 签章、SES V1-V5、印章验证
├── easyofd-convert     🔄 OFD ↔ PDF 转换
├── easyofd-gm          🇨🇳 国密算法（SM2/SM3/SM4）集成
├── easyofd-crypto      🔒 OFD 加密基础设施（SM4、PKCS#12）
├── easyofd-archive     🗄️ 归档合规规则引擎
├── easyofd-graphics2d  🎨 2D 图形抽象层（对应 ofdrw-graphics2d）
├── easyofd-font        🔤 字体管理与嵌入
├── easyofd-tool        🧰 CLI：info / to-markdown / to-pdf / sign / verify / pages / merge
├── easyofd-wasm        🌐 WASM 绑定，浏览器端读取（wasm32 实测编译）
├── easyofd-ffi         🅲 C ABI 绑定（15 个函数，cdylib）
└── easyofd-async       ⚡ 异步门面（spawn_blocking 桥接）
```

完整架构文档见 [docs/easyofd-rust-Architecture.zh_CN.md](docs/easyofd-rust-Architecture.zh_CN.md)。

---

## 快速开始

```toml
[dependencies]
easyofd = "0.1"
```

### 1. 派生宏写入

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
    Invoice { title: "发票 #001".into(), amount: "¥100.00".into() },
    Invoice { title: "发票 #002".into(), amount: "¥200.00".into() },
];

EasyOfd::write::<Invoice>("invoices.ofd")
    .metadata_title("月度发票")
    .do_write(&data)?;
```

### 2. 手动构建页面

```rust
use easyofd::{EasyOfd, TextObject, ImageObject, OfdPage};

let mut page = OfdPage::new(210.0, 297.0); // A4
page.add_text(TextObject::new(20.0, 30.0, "你好 OFD！").size(24.0).bold());
page.add_text(TextObject::new(20.0, 60.0, "普通文本"));
page.add_image(ImageObject::jpeg(150.0, 30.0, 30.0, 30.0, jpeg_bytes));

EasyOfd::write_pages_to("output.ofd", vec![page])?;
```

### 3. 流式写入（大文件）

```rust
use easyofd::{EasyOfd, OfdPage, TextObject};

let file = std::fs::File::create("large.ofd")?;
let mut writer = EasyOfd::stream_writer(file);
for i in 1..=100_000 {
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(10.0, 10.0, format!("第 {i} 页")));
    writer.write_page(page)?;
}
writer.finish()?;
```

### 4. 读取 OFD

```rust
use easyofd::EasyOfd;

// visitor 模式 — 页面不保留在内存中
let visited = EasyOfd::read_pages("input.ofd")
    .page_range(1, 10)
    .do_read(|page_number, page| {
        println!("第 {page_number} 页：{} 个对象", page.content.len());
        Ok(())
    })?;
```

### 5. OFD → Markdown

```rust
use easyofd::EasyOfd;

// 内存中转换
let result = EasyOfd::to_markdown("input.ofd").do_convert()?;
println!("{}", result.markdown);
println!("页数: {}, 损失: {}", result.report.pages_converted, result.report.losses.len());

// 流式写入文件
use std::fs::File;
EasyOfd::to_markdown("input.ofd")
    .convert_to(File::create("output.md")?)?;
```

### 6. 模板填充

```rust
use easyofd::EasyOfd;
use std::collections::HashMap;

let mut data = HashMap::new();
data.insert("name".to_string(), "张三".to_string());
data.insert("amount".to_string(), "¥1,234.00".to_string());

EasyOfd::fill_template("template.ofd", &data)?.save("filled.ofd")?;
```

### 7. 编辑已有 OFD

```rust
use easyofd::{OfdEditor, TextObject, Watermark};

let mut editor = OfdEditor::open("input.ofd")?;
editor.add_text_to_page(0, TextObject::new(10.0, 40.0, "新增文本"))?;
editor.apply_watermarks(&[Watermark::text("机密").position(50.0, 150.0)]);
editor.save("edited.ofd")?;
```

### 8. 电子签章（GB/T 38540）

```rust
use easyofd_signature::{ElectronicSeal, OfdSignatureBuilder, SignatureAlgorithm};

let signed = OfdSignatureBuilder::new("input.ofd")
    .algorithm(SignatureAlgorithm::SM2WithSM3)
    .seal(seal_info) // SealInfo / ElectronicSeal — 详见 examples/07_sign_verify.rs
    .add_signature(secret_key, vec![seal])
    .sign()?; // → SignedOfd
signed.save("signed.ofd")?;
```

> 验签（SES V1/V4/V5 容器、`check_seal_match`、证书链/CRL/OCSP）与篡改检测见
> `crates/easyofd-signature/tests/` 与 `examples/07_sign_verify.rs`。

> 更多完整示例——电子印章（SES）、批量签章、合并、归档检查、加密、关键字搜索——
> 见 `crates/easyofd/examples/`（22 个可运行示例，索引在 [docs/usage-guide.md](docs/usage-guide.md)）。

---

## API 参考

### 入口

所有操作通过 `EasyOfd` 静态方法：

| 方法 | 返回值 | 用途 |
|---|---|---|
| `EasyOfd::write::<T>(path)` | `OfdWriterBuilder<T>` | 使用 `OfdModel` 的类型化写入 |
| `EasyOfd::write_pages(path)` | `PageWriterBuilder` | 手动页面写入 |
| `EasyOfd::write_pages_to(path, pages)` | `OfdResult<()>` | 一次性文件写入 |
| `EasyOfd::write_pages_to_bytes(pages)` | `OfdResult<Vec<u8>>` | 一次性字节写入 |
| `EasyOfd::stream_writer(output)` | `OfdStreamWriter<W>` | 流式写入 |
| `EasyOfd::read(path)` | `OfdResult<OfdReader>` | 完整读取（元数据 + 页面） |
| `EasyOfd::read_from_bytes(data)` | `OfdResult<OfdReader>` | 从内存字节读取 |
| `EasyOfd::read_pages(path)` | `OfdReadBuilder` | 页面 visitor |
| `EasyOfd::to_markdown(path)` | `MarkdownConversionBuilder` | Markdown 转换 |
| `EasyOfd::fill_template(path, data)` | `OfdResult<OfdTemplateFiller>` | 模板填充 |

签章、验签、加密、合并与关键字搜索由专用 crate 提供
（`easyofd-signature`、`easyofd-crypto`、`easyofd-tool`），详见
[docs/usage-guide.md](docs/usage-guide.md)。

### 核心类型

| 类型 | 用途 |
|---|---|
| `OfdPage` | 页面：宽度、高度、内容对象列表 |
| `TextObject` | 定位文本：字体、字号、字重、颜色 |
| `ImageObject` | 定位图片（JPEG/PNG/BMP/TIFF） |
| `PathObject` | 矢量路径（类 SVG `d` 属性） |
| `OfdModel` | 将 Rust 结构体映射为 OFD 页面的 trait |
| `OfdError` | 统一错误枚举（全 workspace `OfdResult<T>`） |
| `ConversionReport` | Markdown 转换结果 + 损失 + 警告 |

---

## 设计原则

| 原则 | 实现 |
|---|---|
| **零 unsafe** | 全 workspace `#![forbid(unsafe_code)]`（FFI crate 豁免，unsafe 块带 `SAFETY` 注释） |
| **流畅 Builder** | `mut self → Self` + `#[must_use]` |
| **编译期反射** | `#[derive(OfdModel)]` — 零运行时开销 |
| **单一入口** | `EasyOfd` — 可发现的静态工厂 |
| **GB/T 33190-2016** | 合规 OFD ZIP + 正确 XML 命名空间 |
| **字节级保真** | raw XML 直通 + 与 ofdrw 的规范化全文 roundtrip（0 偏差） |
| **流式优先** | Writer/Reader/Markdown 均支持逐页处理；合并对每个源 O(1) 内存 |
| **关注点分离** | 每个 crate 一个职责；外观层负责组装 |

---

## Workspace

21 个 crate，4 组（全部以 `0.1.1` 发布至 crates.io）：

| 分组 | Crate |
|---|---|
| 外观与核心 | `easyofd`, `easyofd-core`, `easyofd-derive`, `easyofd-derive-impl` |
| 读写链路 | `easyofd-reader`, `easyofd-writer`, `easyofd-package`, `easyofd-layout`, `easyofd-markdown`, `easyofd-template` |
| ofdrw 对齐模块 | `easyofd-signature`, `easyofd-convert`, `easyofd-gm`, `easyofd-crypto`, `easyofd-archive`, `easyofd-graphics2d`, `easyofd-font` |
| 平台与工具 | `easyofd-tool`（CLI）, `easyofd-wasm`, `easyofd-ffi`, `easyofd-async` |

**2860 测试 · 22 个示例 · clippy `-D warnings` 零警告 · 覆盖率 93%+ · 6 个 CI workflow（3 系统 × 2 工具链）**

---

## 性能对比 ofdrw（Java）

完整基准方法与各场景结果：
[docs/benchmark-report.md](docs/benchmark-report.md)（2026-08-16，Apple Silicon vs OpenJDK 21）。

摘要：18 个写/读/roundtrip 场景中 **比 ofdrw 快 7–59 倍**。Rust 全程内存操作
（无需解压到磁盘、无布局引擎开销）；方法论差异见报告。

---

## 示例

| 示例 | 说明 | 运行 |
|---|---|---|
| `01_hello_ofd` | 最小 OFD 创建 | `cargo run --example 01_hello_ofd` |
| `02_read_metadata` | 读取元数据 / DocInfo 字段 | `cargo run --example 02_read_metadata` |
| `03_text_image_page` | 文本 + 图片 + 矢量页面 | `cargo run --example 03_text_image_page` |
| `04_stream_writer` | 逐页流式写入 | `cargo run --example 04_stream_writer` |
| `05_template_fill` | 模板占位符填充 | `cargo run --example 05_template_fill` |
| `06_to_markdown` | OFD → Markdown + 损失报告 | `cargo run --example 06_to_markdown` |
| `07_sign_verify` | SM2WithSM3 签名 → 验证 | `cargo run --example 07_sign_verify` |
| `08_merge_docs` | 多文档合并 | `cargo run --example 08_merge_docs` |
| `09_archive_check` | 归档合规规则 | `cargo run --example 09_archive_check` |
| `10_convert_pdf` | OFD ↔ PDF 转换 | `cargo run --example 10_convert_pdf` |
| `11_keyword_search` | 跨边界关键字搜索 | `cargo run --example 11_keyword_search` |
| `12_encrypt_decrypt` | SM4 加密 roundtrip | `cargo run --example 12_encrypt_decrypt` |
| `batch_sign` | 批量 + 多签章模式 | `cargo run --example batch_sign` |
| `benchmark` | 性能基准测试 | `cargo run --release --example benchmark -- 10000` |

完整列表（22 个示例）：[docs/usage-guide.md](docs/usage-guide.md)。

---

## 测试

```bash
# 全部测试（2860）
cargo test --workspace

# Clippy 检查
cargo clippy --workspace -- -D warnings

# compile-fail 测试（派生宏错误提示）
cargo test -p easyofd-derive-impl

# ofdrw 字节级一致性（70 个样本）
cargo test -p easyofd --test roundtrip_diff --release
```

---

## 路线图

| 版本 | 里程碑 | 状态 |
|---|---|:---:|
| v0.1.0 | 首次 crates.io 发布（外观层 + 核心） | ✅ 2026-08-10 发布 |
| v0.1.1 | 完整 21-crate 工作区：签章、加密、PDF、合并、WASM/FFI/async；ofdrw 字节级一致 | ✅ 2026-08-21 发布 |

---

## 贡献

1. Fork 并 clone
2. `cargo test --workspace` — 所有测试必须通过
3. `cargo clippy --workspace -- -D warnings` — 无警告
4. 所有新代码必须有 `#[test]` 覆盖
5. 禁止 `unsafe` 代码 — `#![forbid(unsafe_code)]` 强制执行（FFI crate 豁免）

---

## 许可证

Apache-2.0。见 [LICENSE](LICENSE)。
