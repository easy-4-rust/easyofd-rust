# Superpowers 规格体系索引

> easyofd-rust 的规格事实源（对齐 liteflow `docs/superpowers` 结构）。

## 当前状态（2026-08-12）

**全部 7 个历史计划已归档**：76/77 任务完全达标（98.7%），唯一 🟡 项（覆盖率
93.19% vs Phase 0 记录 94.14%）经定案为分母扩大非回归（12→18 crate），非缺陷。
两项核对遗留（unsafe 门禁继承失效、lints 分散）已于 commit `d32ffdc` 修复。

## plans/（历史实施计划，全部 ✅ ARCHIVED）

| 计划 | 内容 | 验证证据 |
|---|---|---|
| [phase0-foundation](plans/2026-08-12-phase0-foundation.md) | v0.1.0~v0.5.0 基础读写/流式写入 | 18 crate workspace |
| [phase1-route1-xml-element-tree](plans/2026-08-12-phase1-route1-xml-element-tree.md) | Route 1 全元素树迁移 | xml_element.rs + 40 类，roundtrip 60/60 |
| [phase2-type-coverage](plans/2026-08-12-phase2-type-coverage.md) | 类型覆盖 99.4%（484/487） | itext_exclusions.rs 文档化 |
| [phase3-bidirectional-verification](plans/2026-08-12-phase3-bidirectional-verification.md) | 双向验证字节级对齐 | roundtrip_diff: 0 ZIP + 0 XML |
| [phase4-p0p4-gap-fill](plans/2026-08-12-phase4-p0p4-gap-fill.md) | P0-P4 缺口补齐 | SES V1/V4/V5 + 5 容器 + 布局引擎 |
| [phase5-production-readiness](plans/2026-08-12-phase5-production-readiness.md) | 生产就绪 | 4 fuzz target + 5 CI workflows |
| [phase6-branch-governance](plans/2026-08-12-phase6-branch-governance.md) | dev/main 分支治理 | 15 Dependabot PR 处理完毕 |

## specs/（活文档）

| 规格 | 内容 |
|---|---|
| [v1.0.0-version-plan-design](specs/2026-08-12-v1.0.0-version-plan-design.md) | v1.0.0 发布定义 + v1.1/v1.2/v2.0 候选方向 |
| [tech-roadmap-status-verification](specs/2026-08-12-tech-roadmap-status-verification.md) | tech-roadmap 旧条目 vs 实际状态核对 |

## 新变更的工作流

1. 变更前：先读本 README 与相关 spec
2. 新计划：`plans/YYYY-MM-DD-<主题>.md`（对齐 liteflow 命名）
3. 完成门禁：任务勾选 ≠ 完成，必须附实际命令输出（test/roundtrip/clippy）
4. 归档：全部任务实测通过后，状态行追加 `ARCHIVED` 并更新本索引
