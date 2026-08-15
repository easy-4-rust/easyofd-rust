# Phase 0: 基础读写与流式写入

> **阶段**: v0.1.0 ~ v0.5.0 基础交付
> **时间跨度**: 项目初期 ~ 2026-08-09
> **状态**: ✅ 已完成（全部任务实测验证通过，2026-08-12 ARCHIVED）

## 目标

建立 easyofd-rust 的核心读写能力，实现 GB/T 33190-2016 基础 OFD 文档操作，包括 Writer、Reader、Template、Package、Signature、Convert、Layout、Markdown 八大模块。

## 范围

- 12 crate workspace 架构（easyofd / core / derive / derive-impl / reader / writer / layout / signature / convert / markdown / template / package）
- `#![forbid(unsafe_code)]` 全 workspace
- Builder 模式 + `#[derive(OfdModel)]` 编译期反射
- 流式写入（StreamWriter）+ 编辑器（Editor）
- 模板占位符替换（OfdTemplateFiller）
- ZIP 安全读取（PackageLimits 防炸弹/路径穿越）
- SM2/SM3 签名基础路径
- OFD → Markdown 转换 + 损失报告
- 简化 PDF ↔ OFD 转换（printpdf / lopdf）

## 方案

### 架构分层

```text
easyofd (facade)
  ├── easyofd-core        类型、trait、错误
  ├── easyofd-derive      #[derive(OfdModel)] 编译期反射
  ├── easyofd-writer      ZIP/XML 生成 + 流式 Writer + Editor
  ├── easyofd-reader      SAX 解析 + 资源提取
  ├── easyofd-layout      版面分析 + 排版
  ├── easyofd-signature   签章 API + SM2 签名
  ├── easyofd-convert     PDF ↔ OFD 转换
  ├── easyofd-markdown    OFD → Markdown
  ├── easyofd-template    模板填充
  └── easyofd-package     ZIP 安全
```

### 技术选型

| 领域 | Rust 组件 | crate |
|---|---|---|
| ZIP 读写 | zip | zip 8.6.0 |
| XML 解析/生成 | quick-xml | quick-xml 0.41.0 |
| 时间处理 | chrono | chrono 0.4.45 |
| 错误处理 | thiserror | thiserror 2.0.18 |
| 派生宏 | syn + quote + proc-macro2 | 1.0.x |

## 任务列表

- [x] 建立 workspace 结构（12 crate）
- [x] 实现 easyofd-core 基础类型（OfdPage / TextObject / ImageObject / PathObject / OfdMetadata）
- [x] 实现 easyofd-derive 派生宏（#[derive(OfdModel)]）
- [x] 实现 easyofd-writer（Builder 模式 + StreamWriter + Editor）
- [x] 实现 easyofd-reader（SAX 解析 + 资源提取）
- [x] 实现 easyofd-template（模板占位符替换）
- [x] 实现 easyofd-package（ZIP 安全 + PackageLimits）
- [x] 实现 easyofd-signature（SM2/SM3 签名基础路径）
- [x] 实现 easyofd-convert（简化 PDF ↔ OFD）
- [x] 实现 easyofd-layout（基础版面分析）
- [x] 实现 easyofd-markdown（OFD → Markdown + 损失报告）
- [x] 实现 easyofd facade（EasyOfd 静态门面）
- [x] 767 测试通过，94.14% 覆盖率

## 验证标准

| 维度 | 标准 | 实际状态 |
|---|---|---|
| 测试 | 全部通过 | ✅ 767 tests (v1.0.0 commit) |
| 覆盖率 | ≥ 90% | ✅ 94.14% |
| Clippy | 零警告 | ✅ |
| unsafe | 禁止 | ✅ `#![forbid(unsafe_code)]` |
| 真实样本 | 5 个 OFD | ✅ tests/fixtures/real_ofd/ |

## 状态

**✅ 已完成** — commit `4647de5` (v1.0.0: full P0-P8 delivery, 19 crates, 767 tests, 94.14% cov)

## 证据

- `Cargo.toml`: workspace 定义 18 crate
- `crates/easyofd/src/lib.rs`: EasyOfd facade
- `crates/easyofd-writer/src/lib.rs`: Builder + StreamWriter + Editor
- `crates/easyofd-reader/src/lib.rs`: SAX 解析
- `crates/easyofd-template/src/lib.rs`: 模板填充
- `crates/easyofd-package/src/lib.rs`: ZIP 安全
- `crates/easyofd-signature/src/ofd_signature_builder.rs`: SM2/SM3 签名
- `crates/easyofd-convert/src/lib.rs`: PDF ↔ OFD
- `crates/easyofd-layout/src/lib.rs`: 版面分析
- `crates/easyofd-markdown/src/lib.rs`: OFD → Markdown
