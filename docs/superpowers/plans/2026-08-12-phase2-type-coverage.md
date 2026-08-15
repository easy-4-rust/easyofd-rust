# Phase 2: 类型覆盖补齐

> **阶段**: ofdrw 类型镜像移植
> **时间跨度**: 2026-08-10 ~ 2026-08-11
> **状态**: ✅ 已完成（全部任务实测验证通过，2026-08-12 ARCHIVED）（99.4%，484/487）

## 目标

将 Java ofdrw 的 ~519 个 public 类型镜像移植到 Rust，实现 99.4% 的类型覆盖率（484/487），建立类型覆盖追踪机制。

## 范围

- `easyofd-core`: 基础值类型（Weight / Point / Position / Rectangle / Span / Border）
- `easyofd-core`: Action / Annotation / Attachment / Versions / Extensions / Bookmark / Permission / CompositeObj / CustomTags 子模块
- `easyofd-gm`: GB/T 35275 sm2_struct（SignedData / ContentInfo / SignerInfo / IssuerAndSerialNumber / Sm2Cipher / OIDs）
- `easyofd-font`: FontName + Font（标准字体名 + 逻辑字体）
- `easyofd-convert`: itext_exclusions.rs（3 个 iText 排除类型文档化）

## 方案

### 移植策略

1. **批量并行**: 3 agent 并行移植 221 类型（151→318/519, 61.3%）
2. **优先级排序**: 按使用频率和公共 API 重要性排序
3. **每类型附测试**: 每个移植的类型都附 roundtrip 测试
4. **别名策略**: 无直接 Rust 等价物的类型用 `pub type` / `pub use` 别名

### 类型覆盖边界

- ofdrw 487 个 unique 类型
- 484 已覆盖（99.4%）
- 3 个排除：ItextFontUtil / ItextTrueTypeFont / Keep（iText 7 Java 依赖）
- 排除理由：iText 7 镜像需要 FFI/JNI 或完整 Rust 移植，违反"纯 Rust"目标

## 任务列表

- [x] 基础值类型移植：Weight / Point / Position / Rectangle
- [x] 文本类型移植：Span / Border / Font / FontName
- [x] GB/T 35275 sm2_struct 移植（SignedData / ContentInfo / SignerInfo 等）
- [x] Action 子模块：14 个类型（hyperlink / goto / sound / movie / uri）
- [x] Annotation 子模块：6 个类型（text / highlight / stamp / popup）
- [x] Attachment 子模块：2 个类型
- [x] Versions 子模块：5 个类型
- [x] Extensions 子模块：3 个类型
- [x] Bookmark / Permission / CompositeObj / CustomTags 子模块
- [x] UserFek / SignIDProvider 等别名类型
- [x] itext_exclusions.rs 文档化（3 个排除类型）
- [x] 类型覆盖 99.4%（484/487）验证

## 验证标准

| 维度 | 标准 | 实际状态 |
|---|---|---|
| 类型覆盖率 | ≥ 99% | ✅ 99.4%（484/487） |
| 排除类型文档 | 全部记录 | ✅ itext_exclusions.rs |
| 每类型测试 | roundtrip | ✅ |
| 用户确认 | 明确接受 99.4% | ✅ commit da8826d |

## 状态

**✅ 已完成** — commits `09e2de2` ~ `da8826d`

## 证据

- `crates/easyofd-core/src/action/`: 14 个 Action 类型文件
- `crates/easyofd-core/src/annotation/`: 7 个 Annotation 类型文件
- `crates/easyofd-core/src/attachment/`: 3 个 Attachment 类型文件
- `crates/easyofd-core/src/versions/`: 6 个 Versions 类型文件
- `crates/easyofd-core/src/extensions/`: 4 个 Extensions 类型文件
- `crates/easyofd-core/src/custom_tags/`: 3 个 CustomTags 类型文件
- `crates/easyofd-core/src/doc/`: bookmark + permission 子模块
- `crates/easyofd-gm/src/sm2_struct/`: GB/T 35275 完整结构
- `crates/easyofd-font/src/font_name.rs` + `font.rs`: 字体类型
- `crates/easyofd-convert/src/itext_exclusions.rs`: 排除文档
- commit `a96a4e3`: 类型覆盖 99.4% (484/487)
- commit `da8826d`: 用户确认 99.4% 覆盖验收
