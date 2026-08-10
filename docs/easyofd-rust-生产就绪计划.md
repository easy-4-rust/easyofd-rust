# easyofd-rust 生产就绪计划

> **目标**：将 easyofd-rust 从 v0.1.0 实验阶段推进至可用于电子发票、公文等生产场景
> **创建日期**：2026-08-10
> **当前状态**：199 测试通过，91.47% 覆盖率，Clippy 通过

---

## Phase 1：代码质量与结构（本次交付）

### 1.1 拆分大文件（>800 行）

| 文件 | 当前行数 | 目标 | 拆分策略 |
|---|---:|---|---|
| `easyofd-writer/src/lib.rs` | 1263 | <800 | 提取 `editor.rs`、`xml_builder.rs` |
| `easyofd-core/src/model.rs` | 866 | <800 | 提取 `watermark.rs`、`page_size.rs` |
| `easyofd-reader/src/lib.rs` | 836 | <800 | 提取 `zip_helper.rs`、`xml_parser.rs` |

### 1.2 GitHub Actions CI

- `ci.yml`：test + clippy + fmt + MSRV check
- `coverage.yml`：cargo-llvm-cov + 上传报告
- `release.yml`：tag 触发 crates.io 发布

### 1.3 提升低覆盖率模块

| Crate | 当前覆盖率 | 目标 | 策略 |
|---|---:|---|---|
| easyofd-package | 78.77% | 85%+ | 补充边界条件测试 |
| easyofd-signature | 83.21% | 90%+ | 补充 API 路径测试 |
| easyofd-markdown | 84.48% | 90%+ | 补充图片/路径转换测试 |

---

## Phase 2：签章密码学实现

### 2.1 SM2/SM3 签名

- 引入纯 Rust SM2/SM3 实现（`sm2` + `sm3` crate）
- 实现 `Signature.xml` 真实摘要计算
- 实现 `SignedInfo` 和 `SignatureValue` 生成

### 2.2 签章验证

- 实现签名验证 API
- 补充签章 roundtrip 测试

---

## Phase 3：OFD 合规性

### 3.1 合规性测试

- 使用真实 OFD 文件（电子发票样本）验证读取
- 验证 GB/T 33190-2016 XML 命名空间
- 验证 ZIP 结构合规性

### 3.2 边界条件

- 超大文件（>100MB）流式处理
- 损坏 ZIP 文件容错
- 非标准 OFD 文件兼容性

---

## Phase 4：发布准备

### 4.1 crates.io 发布

- 补充 `description`、`license`、`repository`、`keywords`
- 设置 `publish = true`（默认）
- 首次发布 v0.1.0

### 4.2 文档

- `cargo doc` 无警告
- 补充 examples 目录
- CHANGELOG.md

### 4.3 版本策略

- v0.1.x：当前功能稳定化
- v0.2.0：签章密码学
- v0.3.0：PDF 互转
- v1.0.0：生产就绪

---

## 验收标准

| 维度 | 标准 |
|---|---|
| 测试 | 全部通过，无 flaky |
| 覆盖率 | ≥ 90% 行覆盖 |
| Clippy | `-D warnings` 零警告 |
| 文件大小 | 所有 `.rs` 文件 ≤ 800 行 |
| CI | GitHub Actions 全绿 |
| 签章 | SM2/SM3 真实签名可验证 |
| 发布 | crates.io 可安装 |
