# Phase 5: 生产就绪

> **阶段**: fuzz / 覆盖率 / panic 降级 / CI
> **时间跨度**: 2026-08-12
> **状态**: ✅ 已完成

## 目标

将 easyofd-rust 从"功能完整"推进至"生产就绪"，建立模糊测试、归档规则覆盖率、panic 降级、CI 矩阵等质量门禁。

## 范围

- **fuzz 设施**: cargo-fuzz 4 个 target + 每日定时 CI
- **归档规则覆盖率**: archive crate 完整测试
- **panic 降级**: 所有 expect/unwrap 审计 + Result 替代
- **CI 矩阵**: ci.yml + coverage.yml + security.yml + fuzz.yml + release.yml

## 方案

### Fuzz 设施

```text
fuzz/
├── Cargo.toml
├── fuzz_targets/
│   ├── fuzz_xml.rs          # XML 解析 fuzz
│   ├── fuzz_ofd_reader.rs   # OFD 读取 fuzz
│   ├── fuzz_ses_der.rs      # SES DER 解析 fuzz
│   └── fuzz_pdf.rs          # PDF 解析 fuzz
├── corpus/                   # 49 个变异 corpus 样本
└── artifacts/                # crash 产物（.gitignore）
```

### CI 矩阵

| Workflow | 触发条件 | 内容 |
|---|---|---|
| ci.yml | push/PR to main/dev | test + clippy + fmt + MSRV check（3 OS × 2 Rust） |
| coverage.yml | push to main + PR + daily | cargo-llvm-cov + Codecov 上传 |
| security.yml | push to main + weekly | cargo-audit + cargo-deny |
| fuzz.yml | daily 03:00 UTC | cargo-fuzz 4 target × 100000 runs |
| release.yml | tag push | crates.io 发布 |

### Panic 降级

- 审计所有 `expect()` / `unwrap()` 调用
- 关键路径改用 `Result` + `?` 传播
- 保留必要的 `expect()` 用于"不可能失败"场景（附注释说明）

## 任务列表

- [x] 建立 fuzz/ 目录 + Cargo.toml
- [x] 实现 4 个 fuzz target（xml / ofd_reader / ses_der / pdf）
- [x] 入库 49 个 fuzzer 变异 corpus 样本
- [x] 新增 fuzz.yml 每日定时模糊测试
- [x] 归档规则覆盖率补齐
- [x] panic 降级审计 + Result 替代
- [x] ci.yml 矩阵配置（3 OS × 2 Rust）
- [x] coverage.yml 配置（cargo-llvm-cov + Codecov）
- [x] security.yml 配置（cargo-audit + cargo-deny）
- [x] release.yml 配置（tag 触发 crates.io 发布）

## 验证标准

| 维度 | 标准 | 实际状态 |
|---|---|---|
| fuzz target | 4 个 | ✅ fuzz_xml / fuzz_ofd_reader / fuzz_ses_der / fuzz_pdf |
| corpus 样本 | ≥ 40 | ✅ 49 个 |
| CI workflow | 5 个 | ✅ ci / coverage / security / fuzz / release |
| MSRV | 1.88 | ✅ ci.yml 矩阵包含 1.88.0 |
| panic 降级 | 关键路径 Result | ✅ |

## 状态

**✅ 已完成** — commits `59d6ce2` ~ `887795c`

## 证据

- `fuzz/fuzz_targets/`: 4 个 fuzz target
- `fuzz/corpus/`: 49 个变异 corpus 样本
- `.github/workflows/ci.yml`: 3 OS × 2 Rust 矩阵
- `.github/workflows/coverage.yml`: cargo-llvm-cov + Codecov
- `.github/workflows/security.yml`: cargo-audit + cargo-deny
- `.github/workflows/fuzz.yml`: 每日 03:00 UTC 4 target × 100000 runs
- `.github/workflows/release.yml`: tag 触发 crates.io 发布
- commit `59d6ce2`: 生产就绪三项补齐
- commit `69c44df`: fuzz.yml 每日定时
- commit `887795c`: 49 个 corpus 样本入库
