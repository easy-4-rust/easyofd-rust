# Phase 6: 分支治理

> **阶段**: dev/main 收敛 + feature 分支合并
> **时间跨度**: 2026-08-11
> **状态**: ✅ 已完成

## 目标

将项目从多分支并行开发收敛为 dev/main 双分支模型，合并 feature/2.0.x 和 feature/3.0.x 分支，建立稳定的发布流程。

## 范围

- feature/2.0.x → dev/main 合并
- feature/3.0.x → dev/main 合并
- dev/main 双分支模型确立
- 依赖更新批次处理

## 方案

### 分支模型

```text
main (稳定发布)
  │
  ├── dev (开发集成)
  │     │
  │     ├── feature/2.0.x (已合并)
  │     └── feature/3.0.x (已合并)
  │
  └── release tags (v1.0.0)
```

### 合并策略

1. feature/2.0.x → dev + main（commit `a710527` + `9a30fd5`）
2. feature/3.0.x → dev + main（commit `f7c0055` + `3c12321`）
3. dev 与 main 保持同步

## 任务列表

- [x] feature/2.0.x 合并到 dev
- [x] feature/2.0.x 合并到 main
- [x] feature/3.0.x 合并到 dev
- [x] feature/3.0.x 合并到 main
- [x] 依赖更新批次处理（ttf-parser / cipher / rand / cbc / criterion 等）
- [x] CI 配置修复（参考 easydoc-rust）

## 验证标准

| 维度 | 标准 | 实际状态 |
|---|---|---|
| 活跃分支 | dev + main | ✅ |
| feature 分支 | 已合并 | ✅ |
| CI 全绿 | main + dev | ✅ |
| 依赖更新 | 最新稳定版 | ✅ |

## 状态

**✅ 已完成** — commits `a710527` ~ `3c12321` + `3c60b2e`

## 证据

- `git log --oneline --all`: 显示 feature/2.0.x 和 feature/3.0.x 已合并
- `git status -sb`: `## dev...origin/dev`（当前在 dev 分支）
- commit `a710527`: Merge branch 'feature/2.0.x' into dev
- commit `9a30fd5`: Merge branch 'feature/2.0.x' into main
- commit `f7c0055`: Merge branch 'feature/3.0.x' into dev
- commit `3c12321`: Merge branch 'feature/3.0.x' into main
- commit `3c60b2e`: ci: 参考 easydoc-rust 修复 GitHub Actions 配置
