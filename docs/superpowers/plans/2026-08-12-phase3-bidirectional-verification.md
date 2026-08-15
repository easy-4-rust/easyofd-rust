# Phase 3: 双向验证与字节级对齐

> **阶段**: roundtrip_diff 60/60 零偏离
> **时间跨度**: 2026-08-10 ~ 2026-08-11
> **状态**: ✅ 已完成（全部任务实测验证通过，2026-08-12 ARCHIVED）

## 目标

建立与 ofdrw 的双向精确比对验证体系，实现 L4 OFD 字节级对齐（roundtrip_diff 60 样本零偏离），确保 easyofd-rust 产出的 OFD 文件与 ofdrw 结构完全一致。

## 范围

- L1 元数据比对（page_count / image_count / path_count / signature_present / text_content_hash）
- L2 XML 结构比对（OFD.xml / Document.xml / Content.xml 命名空间与元素）
- L3 PDF 字节级比对（排除为独立里程碑，用户确认）
- L4 OFD 字节级比对（roundtrip_diff 60/60 零偏离）

## 方案

### 验证管线

```text
OFD 样本 (tests/fixtures/real_ofd/)
  │
  ├── ofdrw_cross_compare.rs    L1 元数据比对
  ├── ofdrw_byte_compare.rs     L2 XML 结构比对
  ├── ofdrw_cross_runner.rs     读取 + roundtrip 验证
  └── roundtrip_diff.rs         L4 字节级比对（60 样本）
```

### 比对维度

| 维度 | 比对方式 | 容差 |
|---|---|---|
| XML 结构 | 语义比对（忽略空白/属性顺序） | 精确 |
| ZIP entry 列表 | 集合比对 | 精确 |
| ZIP entry 内容 | 字节比对 | 精确 |
| 图片数据 | 哈希比对（SHA-256） | 精确 |

### 样本来源

- 55 个 ofdrw 样本（从 ofdrw 仓库复制）
- 5 个基线样本（simple_1 / simple_2 / multi_page_image / signed / with_table）
- 总计 60 个样本

## 任务列表

- [x] 建立 L1 元数据比对（ofdrw_cross_compare.rs，10 tests）
- [x] 建立 L2 XML 结构比对（ofdrw_byte_compare.rs，8 tests）
- [x] 建立 L4 字节级比对（roundtrip_diff.rs）
- [x] 复制 55 个 ofdrw 样本到 tests/fixtures/real_ofd/
- [x] 修复 draw_param_ref PublicRes 怪癖（commit 47b113e）
- [x] 对齐 writer 与 ofdrw 输出约定
- [x] 保留 DocID 通过 roundtrip
- [x] 消除 writer 默认元数据
- [x] roundtrip 全量验证 60/60 零偏离
- [x] L3 PDF 排除为独立里程碑（用户确认）

## 验证标准

| 维度 | 标准 | 实际状态 |
|---|---|---|
| L4 roundtrip_diff | 60/60 零偏离 | ✅ 0 ZIP + 0 XML = 0 |
| L1 元数据比对 | 通过 | ✅ 10 tests |
| L2 XML 结构比对 | 通过 | ✅ 8 tests |
| L3 PDF | 排除 | ✅ 用户确认（独立里程碑） |
| ofdrw_cross_runner | 0 SKIP | ✅ |

## 状态

**✅ 已完成** — commits `47b113e` ~ `da8826d`

## 证据

- `crates/easyofd/tests/roundtrip_diff.rs`: 60 样本零偏离
- `crates/easyofd/tests/ofdrw_cross_compare.rs`: L1 元数据比对
- `crates/easyofd/tests/ofdrw_byte_compare.rs`: L2 XML 结构比对
- `crates/easyofd/tests/ofdrw_cross_runner.rs`: 读取 + roundtrip 验证
- `tests/fixtures/real_ofd/`: 60 个 OFD 样本
- `tests/fixtures/baseline/`: 预期 JSON + XML 基线
- commit `47b113e`: roundtrip 60/60 零偏离
- commit `da8826d`: 用户确认验收
- 实际验证输出：`Total: 0 ZIP diffs + 0 XML diffs = 0 across 60 clean, 0 with deviations, 0 skipped`
