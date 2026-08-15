# Phase 4: P0-P4 缺口补齐

> **阶段**: 密码学真实化 / 布局编排 / 转换渲染 / 类型补全
> **时间跨度**: 2026-08-11
> **状态**: ✅ 已完成（全部任务实测验证通过，2026-08-12 ARCHIVED）

## 目标

补齐 tech-roadmap.md 中 P0-P4 阶段的核心缺口，包括密码学真实化、布局引擎完善、转换渲染增强、签章容器体系等。

## 范围

- **密码学真实化**: SM2/SM3 真实签名 + SES V1/V4/V5 完整结构 + 签名容器体系
- **布局编排**: Div 盒式模型 + SegmentationEngine + StreamingLayoutAnalyzer + VPageParseEngine + DivRender + XY-cut
- **转换渲染**: OFD ↔ PDF 转换增强
- **类型补全**: Action / Annotation / Attachment / Versions / Extensions / Bookmark / Permission / CompositeObj / CustomTags

## 方案

### 密码学真实化

| 组件 | 实现 | 证据 |
|---|---|---|
| SM2/SM3 签名 | sm2/sm3 crate 真实实现 | `ofd_signature_builder.rs` 29 处 SM2 引用 |
| SES V1 | 完整结构（header + esID + property + picture + extDatas + SES_SignInfo） | `easyofd-gm/src/ses/v1.rs` |
| SES V4 | 展平 cert/alg/sig 到 SESeal 顶层 | `easyofd-gm/src/ses/v4.rs` |
| SES V5 | V4 + 可选 timeStamp | `easyofd-gm/src/ses/v5.rs` |
| 签名容器 | GBT35275 / GBT35275-PKCS9 / SES-V1 / SES-V4 / SES-V5 | `sign_containers/` 5 个容器 |
| 骑缝章 | RidingStampPos + CuttingRideStampPos | `stamppos/riding_stamp_pos.rs` |
| 追加签名 | SignMode::ContinueSign | `ofd_signature_builder.rs` |
| PKCS12 | 证书工具 | `easyofd-gm/src/pkcs12_tools.rs` |

### 布局编排

| 组件 | 实现 | 证据 |
|---|---|---|
| Div 盒式模型 | CSS box model 思想 | `div.rs` 379 行 |
| SegmentationEngine | 流式版面分段 | `segment_engine.rs` 222 行 |
| StreamingLayoutAnalyzer | 流式布局分析 | `streaming_layout.rs` 254 行 |
| VPageParseEngine | 虚拟页面 → OFD XML | `vpage_parser.rs` 312 行 |
| DivRender | 渲染分派 | `div_render.rs` 90 行 |
| Processor trait | 渲染处理器接口 | `processor.rs` |
| XY-cut | 页面分割算法 | `xycut.rs` 426 行 |

### 加密与完整性

| 组件 | 实现 | 证据 |
|---|---|---|
| SM4 对称加密 | sm4 crate | `easyofd-crypto/src/sm4.rs` |
| OFD 加解密 | ofd_encrypt / ofd_decryptor | `easyofd-crypto/src/` |
| 用户密码加密 | user_password_encryptor/decryptor | `easyofd-crypto/src/` |
| 用户证书加密 | user_cert_encryptor/decryptor | `easyofd-crypto/src/` |
| 用户 FEK 加密 | user_fek_encryptor/decryptor | `easyofd-crypto/src/` |
| 完整性校验 | integrity 模块 | `easyofd-crypto/src/integrity/` |

## 任务列表

- [x] SM2/SM3 真实签名实现
- [x] SES V1 完整结构实现
- [x] SES V4 实现
- [x] SES V5 实现
- [x] 5 种签名容器实现
- [x] 骑缝章（RidingStampPos + CuttingRideStampPos）
- [x] 追加签名（SignMode::ContinueSign）
- [x] PKCS12 证书工具
- [x] Div 盒式模型实现
- [x] SegmentationEngine 实现
- [x] StreamingLayoutAnalyzer 实现
- [x] VPageParseEngine 实现
- [x] DivRender 实现
- [x] XY-cut 算法实现
- [x] SM4 对称加密实现
- [x] OFD 加解密完整链路
- [x] 完整性校验模块
- [x] Action / Annotation / Attachment / Versions / Extensions / Bookmark / Permission / CompositeObj / CustomTags 子模块

## 验证标准

| 维度 | 标准 | 实际状态 |
|---|---|---|
| 签名容器 | 5 种全部实现 | ✅ GBT35275 / GBT35275-PKCS9 / SES-V1 / SES-V4 / SES-V5 |
| SES 版本 | V1 / V4 / V5 | ✅ |
| 骑缝章 | 实现 | ✅ RidingStampPos + CuttingRideStampPos |
| 布局引擎 | Div + SegmentationEngine + StreamingLayoutAnalyzer + VPageParseEngine + DivRender + XY-cut | ✅ |
| 加密 | SM4 + 完整链路 | ✅ |
| 核心子模块 | 8 个子模块全部实现 | ✅ |

## 状态

**✅ 已完成** — commits `2c8c4fd` ~ `2ab74d2`

## 证据

- `crates/easyofd-gm/src/ses/`: v1.rs / v4.rs / v5.rs
- `crates/easyofd-gm/src/sm2_struct/`: GB/T 35275 完整结构
- `crates/easyofd-signature/src/sign_containers/`: 5 种签名容器
- `crates/easyofd-signature/src/stamppos/`: 骑缝章 + 裁切骑缝章
- `crates/easyofd-layout/src/div.rs`: Div 盒式模型（379 行）
- `crates/easyofd-layout/src/segment_engine.rs`: SegmentationEngine（222 行）
- `crates/easyofd-layout/src/streaming_layout.rs`: StreamingLayoutAnalyzer（254 行）
- `crates/easyofd-layout/src/vpage_parser.rs`: VPageParseEngine（312 行）
- `crates/easyofd-layout/src/div_render.rs`: DivRender（90 行）
- `crates/easyofd-layout/src/xycut.rs`: XY-cut（426 行）
- `crates/easyofd-crypto/src/`: SM4 + 加解密 + 完整性
- `crates/easyofd-core/src/action/`: 14 个 Action 类型
- `crates/easyofd-core/src/annotation/`: 7 个 Annotation 类型
- commit `2c8c4fd`: P0-P4 缺口补齐
- commit `2ab74d2`: 4 项遗留补齐
