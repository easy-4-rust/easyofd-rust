# easyofd-rust 技术选型文档

> **参考来源**：[ddd4j-ddd4r 依赖映射对照表](../../workspace-ddd4r/ddd4r/docs/ddd4j-ddd4r-依赖映射对照表.md)
> **项目基线**：easyofd-rust v0.1.1，Rust 1.88+，Edition 2024，Resolver 3
> **版本**：V1.1.0
> **最后更新**：2026-08-21

---

## 1. 选型原则

| 原则 | 说明 |
|:---|:---|
| 纯 Rust | `#![forbid(unsafe_code)]`，零 C/C++ FFI |
| 最小依赖 | 只引入解决核心问题的 crate |
| 编译期优先 | 派生宏、类型安全、编译期校验 |
| 流式处理 | 大文件不全部加载到内存 |
| GB/T 合规 | GB/T 33190-2016 (OFD)、GB/T 38540 (签章) |

---

## 2. 依赖选型矩阵

### 2.1 核心依赖

| 领域 | Java 对等组件 | Rust 组件 | crate | 版本 | 状态 | 说明 |
|:---|:---|:---|:---|:---|:---:|:---|
| ZIP 读写 | `java.util.zip` / Apache Commons Compress | **zip** | `zip` | 8.6.0 | ✅ | deflate 压缩，ZIP 读写，`ZipWriter`/`ZipArchive` |
| XML 解析 | SAX (`javax.xml.parsers.SAXParser`) | **quick-xml** | `quick-xml` | 0.41.0 | ✅ | SAX 流式解析，内存效率高，支持编码检测 |
| XML 生成 | DOM / JAXB | **quick-xml** | `quick-xml` | 0.41.0 | ✅ | 事件 Writer，不构造完整 DOM 树 |
| 时间处理 | `java.time.LocalDateTime` | **chrono** | `chrono` | 0.4.45 | ✅ | `NaiveDateTime`，零时区依赖 |
| 错误处理 | Java checked exceptions | **thiserror** | `thiserror` | 2.0.18 | ✅ | 派生宏生成 `Error` trait 实现 |
| 序列化 | Jackson / JAXB | **serde** (未来) | `serde` | — | 🗓️ | 当前手动 XML，未来可选 serde 映射 |

### 2.2 派生宏依赖

| 领域 | Java 对等组件 | Rust 组件 | crate | 版本 | 状态 | 说明 |
|:---|:---|:---|:---|:---|:---:|:---|
| 注解处理 | Java Annotation Processing (`@SupportedAnnotationTypes`) | **syn** | `syn` | 3.0.3 | ✅ | Rust 语法树解析 |
| 代码生成 | JavaPoet / JavaWriter | **quote** | `quote` | 1.0.47 | ✅ | Rust 代码生成 |
| Token 操作 | `javax.lang.model.element` | **proc-macro2** | `proc-macro2` | 1.0.107 | ✅ | proc-macro TokenStream 操作 |

### 2.3 安全与包处理

| 领域 | Java 对等组件 | Rust 组件 | crate | 版本 | 状态 | 说明 |
|:---|:---|:---|:---|:---|:---:|:---|
| ZIP 安全 | `java.util.zip.ZipInputStream` + 自定义校验 | **easyofd-package** | — | — | ✅ | ZIP 炸弹防护、路径穿越校验、原子写入 |
| 路径校验 | `java.io.File.getCanonicalPath()` | **std::path** | `std` | — | ✅ | `Component` 枚举逐段校验 |
| 原子写入 | `java.nio.file.Files.move(ATOMIC)` | **std::fs::rename** | `std` | — | ✅ | 同目录临时文件 + rename |

### 2.4 文档格式处理

| 领域 | Java 对等组件 | Rust 组件 | crate | 版本 | 状态 | 说明 |
|:---|:---|:---|:---|:---|:---:|:---|
| OFD 读写 | 自研 OFD SDK | **easyofd-rust** | — | 0.1.1 | ✅ | 21 crate workspace |
| PDF 解析 | Apache PDFBox / iText | **lopdf** | `lopdf` | — | ✅ | PDF → OFD 转换（已实现） |
| PDF 生成 | Apache PDFBox / iText | **printpdf** | `printpdf` | — | ✅ | OFD → PDF 转换（已实现） |
| 图片处理 | `javax.imageio` | **image** (可选) | `image` | — | 🗓️ | 图片格式转换 |

### 2.5 测试依赖

| 领域 | Java 对等组件 | Rust 组件 | crate | 版本 | 状态 | 说明 |
|:---|:---|:---|:---|:---|:---:|:---|
| 单元测试 | JUnit 5 | **#[test]** | `std` | — | ✅ | 内置测试框架 |
| 编译失败测试 | 编译期注解校验 | **trybuild** | `trybuild` | 1.0.116 | ✅ | 派生宏 compile-fail 用例 |
| 基准测试 | JMH | **criterion** (可选) | `criterion` | — | 🗓️ | 性能基准 |
| Mock | Mockito / PowerMock | **mockall** (可选) | `mockall` | — | 🗓️ | trait mock |

---

## 3. 与 ddd4j-ddd4r 映射对照

### 3.1 已采用的 ddd4r 技术栈

| ddd4r 技术选型 | easyofd-rust 是否采用 | 说明 |
|:---|:---:|:---|
| `thiserror` 错误处理 | ✅ | `OfdError` enum + `thiserror` 派生 |
| `chrono` 时间处理 | ✅ | 文档时间戳 |
| `quick-xml` XML 处理 | ✅ | OFD XML 解析与生成 |
| `zip` 文件处理 | ✅ | OFD ZIP 包读写 |
| `serde` 序列化 | 🗓️ | 未来可选，当前手动 XML |
| `tokio` 异步运行时 | ❌ | 同步库，无需异步运行时 |
| `axum` Web 框架 | ❌ | 纯库，无 HTTP 接口 |
| `sqlx` 数据库 | ❌ | 无持久化需求 |
| `tracing` 可观测 | 🗓️ | 未来可选，当前无日志需求 |

### 3.2 easyofd-rust 特有选型

| 技术选型 | ddd4r 对应 | 说明 |
|:---|:---|:---|
| `syn` + `quote` + `proc-macro2` | 无对应（DDD 脚手架不用 proc-macro） | `#[derive(OfdModel)]` 编译期反射 |
| `trybuild` | 无对应 | 派生宏 compile-fail 测试 |
| GB/T 33190-2016 XML 命名空间 | 无对应 | OFD 国标合规 |
| ZIP 炸弹防护 | 无对应 | 文档处理特有安全需求 |

---

## 4. 不采用的技术及原因

| 技术 | 不采用原因 |
|:---|:---|
| `tokio` / `async` | OFD 是文件操作，同步 I/O 足够；避免运行时开销 |
| `serde` + `serde_xml` | OFD XML 结构复杂（命名空间、属性映射），手动 SAX 更可控 |
| `dom` 解析器 (`roxmltree` 等) | 大文件内存不可控，SAX 流式解析更安全 |
| `anyhow` | 库 crate 应用结构化错误 (`thiserror`)，不用动态错误 |
| `rayon` 并行 | 单文件顺序处理，并行收益有限 |
| `regex` | OFD XML 结构化解析，不需要正则 |

---

## 5. 依赖版本锁定策略

| 策略 | 说明 |
|:---|:---|
| Workspace 统一版本 | 所有 crate 通过 `[workspace.dependencies]` 统一版本 |
| 最小 features | `zip` 只启用 `deflate`，`chrono` 只启用 `clock` + `std` |
| 无默认 feature 滥用 | 每个 crate 明确声明所需 feature |
| MSRV 锁定 | `rust-version = "1.88"`，CI 验证 |

---

## 6. 安全与合规

| 维度 | 实现 |
|:---|:---|
| `unsafe` 策略 | `#![forbid(unsafe_code)]` 全 workspace |
| ZIP 安全 | `PackageLimits` 限制条目数、解压大小、压缩比 |
| 路径安全 | `validate_entry_name` 阻止 `..` 和绝对路径 |
| XML 安全 | SAX 解析，不加载外部实体 |
| 原子写入 | 同目录临时文件 + `rename` |

---

## 7. 构建与质量门禁

| 门禁 | 工具 | 配置 |
|:---|:---|:---|
| 编译检查 | `cargo check --workspace` | default features |
| 测试 | `cargo test --workspace` | 2860 测试 |
| Lint | `cargo clippy --workspace` | `pedantic` = warn |
| 格式 | `cargo fmt --check` | stable rustfmt |
| 文档 | `cargo doc --workspace` | `missing_docs` = warn |
| 覆盖率 | `cargo-llvm-cov` | 行覆盖率 + 分支覆盖率 |
| Compile-fail | `trybuild` | 2 派生宏错误用例 |

---

## 8. 版本记录

| 版本 | 日期 | 变更说明 |
|:---|:---|:---|
| V1.0.0 | 2026-08-10 | 初始版本；基于 ddd4j-ddd4r 映射表编写 |
| V1.1.0 | 2026-08-21 | 基线升级至 v0.1.1；21 crate workspace；PDF 转换已实现；测试数更新为 2860 |
