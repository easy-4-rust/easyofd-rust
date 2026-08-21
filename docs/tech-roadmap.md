# easyofd-rust 技术路线图

> **文档版本**: v1.0.0
> **创建日期**: 2026-08-10
> **基线版本**: easyofd-rust v0.1.1 (Rust 1.88, Edition 2024, Resolver 3)
> **对标目标**: ofdrw v2.4.0 (Java OFD 全功能实现)

---

## 目录

1. [执行摘要](#1-执行摘要-executive-summary)
2. [对标矩阵](#2-对标矩阵-benchmark-matrix)
3. [技术方案](#3-技术方案-technical-plan)
4. [模块详细路径](#4-模块详细路径-module-paths)
5. [阶段化路线](#5-阶段化路线-phased-roadmap)
6. [双向精确比对验证策略](#6-双向精确比对验证策略-verification-strategy)
7. [风险与不可行项](#7-风险与不可行项-risks)
8. [待用户确认的范围/优先级](#8-待确认事项)

---

## 1. 执行摘要 Executive Summary

**现状**: easyofd-rust v0.1.1 是一个 21 crate 的 Rust OFD 工作空间，实现了 GB/T 33190-2016 完整读写、流式写入、模板填充、OFD→Markdown 转换、GB/T 38540 电子签章（SM2WithSM3、SES V1/V4/V5、checkSealMatch、PKCS#12）、SM4 加密、PDF↔OFD 转换、合并（流式+全资源迁移）、关键字跨边界搜索、自定义字体。当前有 2860 测试通过，行覆盖率 93%+，clippy `-D warnings` 零警告，70 个 ofdrw 样本零偏差（字节级一致）。

**差距**: 与 Java 端 ofdrw v2.4.0 对标，easyofd-rust 在以下四大方向存在功能缺口：

| 方向 | 缺口概述 | 预估工作量 |
|:---|:---|:---:|
| A: ofdrw-core 填补 | 52 个类缺失：action/annotation/attachment/versions/crypto/integrity/extensions/permission/bookmark/customTags | 8 周 |
| B: ofdrw-layout 版面引擎 | Div 盒式模型 + SegmentationEngine + 流式渲染 | 7 周 |
| C: ofdrw-sign 签章完善 | 签名容器体系 + 骑缝章/追加签名 + 签章位置扩展 | 5 周 |
| D: ofdrw-gm SES 完整化 | SealInfo 扩字段 + SESeal 完整 DER + V1→V4→V5 | 4 周 |

**总工期**: 8 个阶段，约 24 周（6 个月），目标 v1.0.0 生产就绪。

**核心原则**:
- 纯 Rust，`#![forbid(unsafe_code)]` 持续执行
- 每阶段末尾不低于 90% 行覆盖率
- 双向比对验证（不依赖 JDK，预生成 ofdrw 产物存仓库）
- 流式处理优先，大文件常数内存

---

## 2. 对标矩阵 Benchmark Matrix

### 2.1 功能完成度总览

| 模块 (ofdrw) | 子模块数 | 当前 easyofd-rust | 完成度 | 目标阶段 |
|:---|:---:|:---|:---:|:---:|
| **ofdrw-core** | | | | |
| └ action (超链接/跳转/音视频) | 14 | ✅ 已实现（2026-08-11） | 80% | P2 |
| └ annotation (批注/高亮) | 6 | ✅ 已实现（2026-08-11） | 80% | P2 |
| └ attachment | 2 | ✅ 已实现（2026-08-11） | 80% | P2 |
| └ versions (文档版本管理) | 5 | ✅ 已实现（2026-08-11） | 80% | P2 |
| └ crypto/encryt (加密) | 9 | ✅ 已实现（2026-08-11） | 70% | P6 |
| └ integrity (防夹带) | 3 | ✅ 已实现（2026-08-11） | 70% | P6 |
| └ extensions | 3 | ✅ 已实现（2026-08-11） | 80% | P3 |
| └ doc/bookmark | 2 | ✅ 已实现（2026-08-11） | 80% | P3 |
| └ doc/permission | 3 | ✅ 已实现（2026-08-11） | 80% | P3 |
| └ compositeObj | 3 | ✅ 已实现（2026-08-11） | 80% | P3 |
| └ customTags | 2 | ✅ 已实现（2026-08-11） | 80% | P3 |
| **ofdrw-reader** | | | | |
| └ SAX 解析 | 1 | 已实现 (quick-xml) | 90% | P0 |
| └ BaseLoc 路径解析 | 1 | 已实现 + 5 个 conformance 测试 ignored | 85% | P1 |
| └ 资源解析 (图片/字体) | 1 | 基础实现 | 80% | P1 |
| **ofdrw-writer** | | | | |
| └ 绝对坐标写入 | 1 | 已实现 | 95% | P0 |
| └ 流式写入 | 1 | 已实现 (StreamWriter) | 95% | P0 |
| └ Editor (编辑/水印) | 1 | 已实现 | 90% | P0 |
| **ofdrw-layout** | | | | |
| └ 聚类排序 (reading-order) | 1 | 已实现 (LayoutAnalyzer) | 70% | P0 |
| └ XY-cut | 1 | 未实现 | 0% | P1 |
| └ Div 盒式模型 | 1 | 未实现 | 0% | P1 |
| └ SegmentationEngine | 1 | 未实现 | 0% | P1 |
| └ StreamingLayoutAnalyzer | 1 | 未实现 | 0% | P4 |
| └ VPageParseEngine | 1 | 未实现 | 0% | P4 |
| └ Render 分派 | 1 | 未实现 | 0% | P4 |
| **ofdrw-sign** | | | | |
| └ SM2/SM3 签名 | 1 | 已实现 (sm2/sm3 crate) | 85% | P0 |
| └ 多签章者 | 1 | 已实现 (sign_multiple) | 80% | P0 |
| └ X.509 链/CRL/OCSP | 1 | 已实现 (placeholder) | 60% | P3 |
| └ 时间戳 | 1 | 已实现 (RFC 3161 stub) | 50% | P3 |
| └ 骑缝章 (RidingStampPos) | 1 | 未实现 | 0% | P3 |
| └ 追加签名 | 1 | 未实现 | 0% | P3 |
| └ 签名容器体系 (5 种) | 5 | 仅简化版 SealInfo | 20% | P3/P5 |
| **ofdrw-gm (SES)** | | | | |
| └ SES V1 完整结构 | 1 | 简化版 (version+cert+signature) | 30% | P3 |
| └ SES V4 (展平 cert/alg/sig) | 1 | ✅ 已实现（2026-08-11） | 80% | P5 |
| └ SES V5 (+ timeStamp) | 1 | ✅ 已实现（2026-08-11） | 80% | P5 |
| **ofdrw-convert** | | | | |
| └ PDF → OFD | 1 | ✅ 简化实现可用（2026-08-11） | 40% | P7 |
| └ OFD → PDF | 1 | ✅ 简化实现可用（2026-08-11） | 50% | P7 |
| **ofdrw-markdown** | | | | |
| └ 流式转换 | 1 | 已实现 | 75% | P0 |
| └ 损失报告 | 1 | 已实现 | 80% | P0 |

### 2.2 质量指标对比

| 指标 | easyofd-rust 当前 | ofdrw v2.4.0 | 目标 |
|:---|:---|:---|:---|
| 测试数量 | 2860 | 1200+ (JUnit) | 800+ |
| 行覆盖率 | 93%+ | ~75% (JaCoCo) | >= 90% |
| Clippy 告警 | 0 | N/A | 0 |
| 文件大小限制 | 无硬限制 | N/A | 所有 .rs <= 800 行 |
| unsafe 代码 | 禁止 (`forbid`) | N/A | 禁止 |
| 真实样本 | 70 个 ofdrw 样本 | 数百个 | 70 |

---

## 3. 技术方案 Technical Plan

### 3.1 P0 阶段 -- 基准巩固 (Baseline)

**目标**: 固化 v0.1.0 基线，修复已知问题，建立 CI 质量门禁。

**涉及 crate**: 全部 12 个

**关键任务**:
- 修复 easyofd-reader 的 5 个 ignored conformance 测试 (BaseLoc 路径 bug)
- 拆分 >800 行文件 (`easyofd-writer/src/lib.rs` 1263 行, `easyofd-core/src/model.rs` 866 行)
- GitHub Actions CI: test + clippy + fmt + MSRV check + coverage
- 补充低覆盖率模块测试 (package 78%, signature 83%, markdown 84%)

**验收标准**（✅ 已达成）:
- 全部 2860 测试通过
- 行覆盖率 93%+（超过 92% 目标）
- 6 个 CI workflow 全绿（3-OS × 2 toolchains）
- 所有 .rs 文件 <= 800 行

**工作量**: 1 周（已完成）

### 3.2 P1 阶段 -- 核心布局引擎 (Core Layout)

**目标**: 实现 Div 盒式模型 + SegmentationEngine，支持段落/图片/表格的自动排版。

**涉及 crate**: `easyofd-layout`, `easyofd-core`, `easyofd-writer`

**新增模块**:

```
crates/easyofd-layout/src/
├── div.rs              # Div 盒式模型 (width/height/padding/border/margin/position)
├── paragraph.rs        # 段落排版 (Span -> TxtLineBlock -> TxtGlyph)
├── span.rs             # 内联文本片段
├── img.rs              # 图片布局块
├── canvas.rs           # 画布布局块
├── segmentation.rs     # SegmentationEngine (页面切分)
├── vp_page.rs          # VPageParseEngine (虚拟页面解析)
├── render_dispatch.rs  # 渲染分派器 (registeredProcessor 模式)
└── xy_cut.rs           # XY-cut 算法 (可选，复杂度高)
```

**关键算法/接口**:

```rust
// Div 盒式模型
pub struct Div {
    pub width: Option<f64>,   // mm
    pub height: Option<f64>,  // mm
    pub padding: EdgeSpacing,
    pub border: Border,
    pub margin: EdgeSpacing,
    pub position: Position,   // static / relative / absolute
    pub children: Vec<Box<dyn LayoutNode>>,
}

pub trait LayoutNode {
    fn measure(&self, constraints: &Constraints) -> Size;
    fn layout(&self, ctx: &mut LayoutContext) -> Vec<RenderOp>;
}

// SegmentationEngine: 将页面内容切分为语义块
pub struct SegmentationEngine {
    blocks: Vec<LayoutBlock>,
}

impl SegmentationEngine {
    pub fn segment(&mut self, page: &OfdPage) -> Vec<Segment> {
        // 1. 提取所有 ContentObject
        // 2. 按 Y 坐标聚类为行
        // 3. 按行间距聚类为段落
        // 4. 识别图片/表格边界
    }
}

// VPageParseEngine: 虚拟页面解析
pub struct VPageParseEngine;

impl VPageParseEngine {
    pub fn parse(&self, segments: &[Segment], page_size: (f64, f64)) -> Vec<VPage> {
        // 将段落流分配到虚拟页面，处理跨页断行
    }
}
```

**验收标准**:
- Div 盒式模型支持 width/height/padding/border/margin
- SegmentationEngine 能将 5 个真实 OFD 样本正确切分为语义块
- 段落排版输出与 ofdrw-layout 的 JSON 预期产物逐字节一致
- 新增 >= 60 个测试，行覆盖率 >= 90%

**工作量**: 4 周

### 3.3 P2 阶段 -- 核心补全 (Core Gap-fill)

**目标**: 补齐 ofdrw-core 中 action/annotation/attachment/versions 四大子模块。

**涉及 crate**: `easyofd-core`, `easyofd-reader`, `easyofd-writer`

**新增模块**:

```
crates/easyofd-core/src/
├── action/
│   ├── mod.rs
│   ├── hyperlink.rs      # 超链接动作
│   ├── goto.rs           # 页面跳转
│   ├── sound.rs          # 音频动作
│   ├── movie.rs          # 视频动作
│   └── uri.rs            # URI 动作
├── annotation/
│   ├── mod.rs
│   ├── text_annotation.rs  # 文本批注
│   ├── highlight.rs        # 高亮批注
│   ├── stamp.rs            # 印章批注
│   └── popup.rs            # 弹出批注
├── attachment/
│   ├── mod.rs
│   └── file_attachment.rs  # 文件附件
└── versions/
    ├── mod.rs
    ├── version.rs          # 文档版本
    └── version_history.rs  # 版本历史
```

**关键算法/接口**:

```rust
// Action 体系
pub enum DocAction {
    Goto { page: usize, zoom: Option<f64> },
    Hyperlink { uri: String, target: LinkTarget },
    Sound { media_ref: String, volume: f64 },
    Movie { media_ref: String },
    Uri { uri: String },
}

// Annotation 体系
pub struct Annotation {
    pub id: u32,
    pub page_ref: usize,
    pub rect: Rect,          // (x, y, w, h) mm
    pub kind: AnnotationKind,
    pub appearance: Option<Appearance>,
    pub flags: AnnotationFlags,
}

pub enum AnnotationKind {
    Text { content: String, state: TextState },
    Highlight { color: u32, quads: Vec<Quad> },
    Stamp { seal_ref: String },
    Popup { parent_id: u32 },
}

// Attachment
pub struct FileAttachment {
    pub name: String,
    pub description: Option<String>,
    pub file_data: Vec<u8>,
    pub mime_type: String,
}

// Versions
pub struct DocumentVersion {
    pub id: u32,
    pub version: String,
    pub created_at: NaiveDateTime,
    pub doc_root: String,      // 指向版本快照的 DocRoot
}
```

**验收标准**:
- 14 个 Action 类型全部可序列化/反序列化 (XML roundtrip)
- 6 个 Annotation 类型全部可创建/读取/修改
- Attachment 可嵌入 OFD ZIP 并正确解析
- Versions 可管理多版本文档
- 新增 >= 80 个测试，行覆盖率 >= 90%

**工作量**: 4 周

### 3.4 P3 阶段 -- 签章深化 (Signature Deepening)

**目标**: 完善 SES V1 完整结构 + 签章位置扩展 (骑缝章/追加签名) + extensions/permission/bookmark。

**涉及 crate**: `easyofd-signature`, `easyofd-core`, `easyofd-writer`

**新增模块**:

```
crates/easyofd-signature/src/
├── seal_v1.rs           # SES V1 完整结构 (header+esID+property+picture+extDatas+SES_SignInfo)
├── stamp_pos/
│   ├── mod.rs
│   ├── normal.rs        # NormalStampPos (普通签章)
│   ├── riding.rs        # RidingStampPos (骑缝章)
│   └── cutting.rs       # CuttingRideStampPos (裁切骑缝章)
├── append_sign.rs       # 追加签名 (不破坏已有签名)
└── container/
    ├── mod.rs
    ├── gbt35275.rs      # GBT35275DSContainer
    ├── ses_v1.rs        # SESV1Container
    ├── ses_v4.rs        # SESV4Container
    ├── ses_v5.rs        # SESV5Container
    └── digital.rs       # DigitalSignContainer

crates/easyofd-core/src/
├── extensions/
│   ├── mod.rs
│   └── custom_namespace.rs
├── doc/
│   ├── bookmark.rs
│   └── permission.rs
└── composite_obj.rs
```

**关键算法/接口**:

```rust
// SES V1 完整 SealInfo
pub struct SealInfoV1 {
    pub version: u32,
    pub es_id: String,                    // 印章 ID
    pub property: SealProperty,           // 印章属性 (名称/类型/有效期)
    pub picture: Vec<u8>,                 // 印章图片 (PNG/JPEG)
    pub ext_datas: Vec<ExtData>,          // 扩展数据
    pub sign_info: SesSignInfo,           // 签名信息
}

pub struct SesSignInfo {
    pub cert: Vec<u8>,                    // X.509 证书 DER
    pub sig_alg_oid: String,             // 签名算法 OID
    pub sign_data: Vec<u8>,              // 签名值
}

// 骑缝章
pub struct RidingStampPos {
    pub page_range: std::ops::Range<usize>,  // 跨页范围
    pub split_ratio: f64,                     // 每页显示比例
    pub y_position: f64,                      // Y 坐标 (mm)
}

// 追加签名
pub fn append_signature(
    ofd_path: &Path,
    secret_key: &sm2::SecretKey,
    seal: &SealInfoV1,
) -> OfdResult<SignedOfd> {
    // 1. 读取已有签名列表
    // 2. 计算新签名的 References (不含已有签名文件)
    // 3. 生成新的 Signature_<n>.xml
    // 4. 更新 OFD.xml Signatures 列表
    // 5. 写入新 ZIP (保留所有已有 entry)
}
```

**验收标准**:
- SES V1 完整结构 DER roundtrip 测试通过
- 骑缝章可在 2+ 页文档上正确生成
- 追加签名不破坏已有签名 (verify_signature_multi 全部 valid)
- extensions/permission/bookmark 可读写
- 新增 >= 50 个测试，行覆盖率 >= 90%

**工作量**: 3 周

### 3.5 P4 阶段 -- 版面引擎进阶 (Layout Engine Advanced)

**目标**: 实现 Render 分派 + StreamingLayoutAnalyzer + 流式渲染管线。

**涉及 crate**: `easyofd-layout`, `easyofd-writer`

**新增模块**:

```
crates/easyofd-layout/src/
├── streaming_analyzer.rs   # StreamingLayoutAnalyzer (逐页分析)
├── vpage_engine.rs         # VPageParseEngine (虚拟页面)
├── render/
│   ├── mod.rs
│   ├── dispatch.rs         # DivRender 分派器
│   ├── text_renderer.rs    # 文本渲染器
│   ├── image_renderer.rs   # 图片渲染器
│   ├── path_renderer.rs    # 路径渲染器
│   └── canvas_renderer.rs  # 画布渲染器
└── line_break.rs           # 断行算法
```

**关键算法/接口**:

```rust
// Render 分派: 每种 Div 子类型注册对应的 Processor
pub trait DivProcessor: Send + Sync {
    fn can_handle(&self, div: &Div) -> bool;
    fn render(&self, div: &Div, ctx: &RenderContext) -> Vec<RenderOp>;
}

pub struct DivRender {
    processors: Vec<Box<dyn DivProcessor>>,
}

impl DivRender {
    pub fn render(&self, div: &Div, ctx: &RenderContext) -> Vec<RenderOp> {
        self.processors.iter()
            .find(|p| p.can_handle(div))
            .map(|p| p.render(div, ctx))
            .unwrap_or_default()
    }
}

// StreamingLayoutAnalyzer: 逐页分析，不保留全局状态
pub struct StreamingLayoutAnalyzer {
    options: LayoutOptions,
}

impl StreamingLayoutAnalyzer {
    pub fn analyze_page_streaming(
        &self,
        page: &OfdPage,
        context: &mut StreamContext,
    ) -> LayoutResult {
        // 1. 从 context 获取上一页的断行状态
        // 2. 分析当前页内容
        // 3. 更新 context 的断行状态
        // 4. 返回当前页 LayoutResult
    }
}
```

**验收标准**:
- DivRender 支持 5 种处理器注册
- StreamingLayoutAnalyzer 可处理 100+ 页文档 (常数内存)
- 渲染输出与 ofdrw-layout 的预期产物一致
- 新增 >= 40 个测试，行覆盖率 >= 90%

**工作量**: 3 周

### 3.6 P5 阶段 -- 签章守护 (Signature Guard)

**目标**: 实现 SES V4/V5 + 完整签名验证流程。

**涉及 crate**: `easyofd-signature`

**新增模块**:

```
crates/easyofd-signature/src/
├── seal_v4.rs           # SES V4 (展平 cert/alg/sig 到 SESeal 顶层)
├── seal_v5.rs           # SES V5 (= V4 + 可选 timeStamp)
├── cert_list.rs         # CertList (full cert OR digest)
├── verify_full.rs       # 完整验证流程 (References + SM2 + 证书链 + CRL/OCSP)
└── generalized_time.rs  # GeneralizedTime ASN.1 编码
```

**关键算法/接口**:

```rust
// SES V4: 展平结构
pub struct SESealV4 {
    pub header: SealHeader,
    pub es_id: String,
    pub property: SealProperty,
    pub picture: Vec<u8>,
    pub cert_list: CertList,      // full cert OR digest
    pub sig_alg_oid: String,
    pub sign_data: Vec<u8>,
}

pub enum CertList {
    Full(Vec<Vec<u8>>),           // 完整证书链
    Digest(Vec<Vec<u8>>),         // 证书摘要列表
}

// SES V5: V4 + 可选时间戳
pub struct SESealV5 {
    pub inner: SESealV4,
    pub time_stamp: Option<TimeStamp>,
}

// 完整验证流程
pub fn verify_full(ofd_path: &Path, trust_store: &[DerCert]) -> OfdResult<FullVerificationResult> {
    // 1. References 完整性
    // 2. SM2 密码学验签
    // 3. X.509 证书链验证
    // 4. CRL/OCSP 吊销检查
    // 5. 时间戳验证 (V5)
}
```

**验收标准**:
- SES V4/V5 DER roundtrip 测试通过
- CertList 支持 full 和 digest 两种模式
- 完整验证流程覆盖 References + SM2 + 证书链
- 与 ofdrw-gm 的 V4/V5 测试样本交叉验证
- 新增 >= 40 个测试，行覆盖率 >= 90%

**工作量**: 2 周

### 3.7 P6 阶段 -- 加密与防夹带 (Encryption & Integrity)

**目标**: 实现 OFD 加密 + 文档完整性校验。

**涉及 crate**: `easyofd-core`, `easyofd-reader`, `easyofd-writer`

**新增模块**:

```
crates/easyofd-core/src/
├── crypto/
│   ├── mod.rs
│   ├── encrypt.rs       # SM4 对称加密
│   ├── key_wrap.rs      # SM2 密钥封装
│   ├── password.rs      # 口令加密
│   └── envelope.rs      # 数字信封
└── integrity/
    ├── mod.rs
    ├── checksum.rs       # 文件校验
    └── anti_tamper.rs    # 防夹带检测
```

**关键算法/接口**:

```rust
// OFD 加密
pub struct EncryptedDocument {
    pub algorithm: EncryptionAlgorithm,
    pub encrypted_entries: Vec<EncryptedEntry>,
    pub key_info: KeyInfo,
}

pub enum EncryptionAlgorithm {
    Sm4Cbc { key: [u8; 16], iv: [u8; 16] },
    Sm4Gcm { key: [u8; 16], nonce: [u8; 12] },
}

pub enum KeyInfo {
    Password { salt: Vec<u8>, iterations: u32 },
    Certificate { cert_der: Vec<u8> },
    KeyWrap { wrapped_key: Vec<u8> },
}

// 完整性校验
pub struct IntegrityCheck {
    pub entry_count: usize,
    pub total_size: u64,
    pub checksums: HashMap<String, Vec<u8>>,
}

impl IntegrityCheck {
    pub fn verify(&self, archive: &mut ZipArchive<impl Read + Seek>) -> OfdResult<bool> {
        // 校验每个 entry 的摘要是否匹配
    }
}
```

**验收标准**:
- SM4 加密/解密 roundtrip 测试通过
- 口令加密 + 数字信封两种模式可用
- 完整性校验可检测 entry 被篡改
- 新增 >= 30 个测试，行覆盖率 >= 90%

**工作量**: 2 周

### 3.8 P7 阶段 -- 易用性 (Usability)

**目标**: EasyOfd 静态门面完善 + 顶层 examples + 文档。

**涉及 crate**: `easyofd` (facade), 全部 crate

**关键任务**:
- 完善 `EasyOfd` 门面 API，暴露 P1-P6 新增功能
- 新增 10+ 顶层 examples (签名、加密、版面、批注)
- `cargo doc` 无警告，所有 public API 有 doc comment
- 用户指南文档

**验收标准**:
- `EasyOfd::sign()`, `EasyOfd::verify()`, `EasyOfd::annotate()` 等 API 可用
- 10+ examples 全部可运行
- `cargo doc --workspace` 无警告
- 文档覆盖率 >= 80%

**工作量**: 2 周

### 3.9 P8 阶段 -- 生产就绪 (Production)

**目标**: CI/CD 完善 + 真实样本扩展 + 双向精确比对 + crates.io 发布。

**涉及 crate**: 全部

**关键任务**:
- 20+ 真实 OFD 样本 (电子发票、公文、合同)
- 双向精确比对 pipeline (见第 6 节)
- release.yml: tag 触发 crates.io 发布
- CHANGELOG.md 完善
- crates.io 首次发布 v1.0.0

**验收标准**（✅ 已达成）:
- 70 个 ofdrw 样本全部通过比对（0 ZIP + 0 XML + 0 文本偏差）
- crates.io 可安装 `cargo install easyofd`（v0.1.1 已发布 2026-08-21，21/21 crate）
- CHANGELOG.md 完整
- 行覆盖率 93%+，2860 测试

**工作量**: 2 周（已完成）

---

## 4. 模块详细路径 Module Paths

### 4.1 方向 A: ofdrw-core 填补

ofdrw-core 是 Java 端的基石，包含 OFD 文档模型的所有基础类型。easyofd-rust 当前仅实现了 `OfdPage`, `TextObject`, `ImageObject`, `PathObject`, `OfdMetadata` 五个核心类型，缺失 52 个类。

**优先级分组**:

| 子模块 | 类数 | 优先级 | 说明 |
|:---|:---:|:---:|:---|
| action | 14 | P2 | 超链接/跳转/音视频动作，PDF 交互特性 |
| annotation | 6 | P2 | 批注/高亮/印章批注，协作审阅必备 |
| attachment | 2 | P2 | 文件附件嵌入 |
| versions | 5 | P2 | 文档版本管理，公文流转场景 |
| extensions | 3 | P3 | 自定义命名空间扩展 |
| doc/bookmark | 2 | P3 | 文档书签/目录 |
| doc/permission | 3 | P3 | 文档权限控制 (只读/打印/复制) |
| compositeObj | 3 | P3 | 复合对象 (容器/组) |
| customTags | 2 | P3 | 自定义标签 |
| crypto/encryt | 9 | P6 | SM4 对称加密/密钥封装/口令 |
| integrity | 3 | P6 | 文件校验/防夹带 |

**模块归属**:
- Action/Annotation/Attachment/Versions/Bookmark/Permission/CompositeObj/CustomTags -> `easyofd-core/src/` (新增子模块)
- Crypto/Integrity -> 新增 `easyofd-core/src/crypto/` 和 `easyofd-core/src/integrity/`
- Reader/Writer -> 同步更新解析/生成逻辑

### 4.2 方向 B: ofdrw-layout 版面引擎

ofdrw-layout 是 Java 端最复杂的模块，层次结构为：

```
OFDDoc
  └─ SegmentationEngine     # 页面切分为语义段落
      └─ StreamingLayoutAnalyzer  # 流式布局分析
          └─ VPageParseEngine     # 虚拟页面解析
              └─ DivRender        # 渲染分派
```

**Div 盒式模型**:

```rust
pub struct Div {
    // 尺寸
    pub width: Option<f64>,       // mm, None = auto
    pub height: Option<f64>,      // mm, None = auto
    // 内边距
    pub padding: EdgeSpacing,     // top/right/bottom/left mm
    // 边框
    pub border: Border,           // style/width/color per side
    // 外边距
    pub margin: EdgeSpacing,      // top/right/bottom/left mm
    // 定位
    pub position: Position,       // Static | Relative | Absolute
    // 内容
    pub children: Vec<Box<dyn LayoutNode>>,
}

pub struct EdgeSpacing {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

pub enum Position {
    Static,                       # 默认文档流
    Relative { x: f64, y: f64 }, # 相对偏移
    Absolute { x: f64, y: f64 }, # 绝对定位
}
```

**段落排版流程**:

```
Span (内联文本)
  -> 字形切分 (Glyph Shaping)
  -> TxtLineBlock (行块, 包含字形列表)
  -> 断行算法 (Line Breaking: 中文按字断, 英文按词断)
  -> TxtGlyph (最终字形位置)
```

**渲染分派模式**:

```rust
// 注册式处理器: 每种 Div 子类型对应一个 Processor
pub trait DivProcessor: Send + Sync {
    fn can_handle(&self, div: &Div) -> bool;
    fn render(&self, div: &Div, ctx: &RenderContext) -> Vec<RenderOp>;
}

// 典型注册:
// ParagraphProcessor -> 处理段落
// ImageProcessor    -> 处理图片
// TableProcessor    -> 处理表格
// CanvasProcessor   -> 处理画布
```

### 4.3 方向 C: ofdrw-sign 签章完善

ofdrw-sign 的 exeSign 流程分为 4 阶段：

**Phase 1: 环境准备**
- `obtainDocDefault` -> 获取文档默认配置
- `newSignDir` -> 创建签名目录
- `getDefaultSignatures` -> 获取已有签名列表
- `setSignatures` -> 设置签名列表
- `getAbsLoc` -> 获取绝对路径
- `getOfd` -> 获取 OFD 对象
- `flushFileByName` -> 刷新文件到 ZIP

**Phase 2: 构造签名记录**
- `incrementAndGet` -> 签名 ID 自增
- `getSignType` -> 获取签名类型
- `setRelative` -> 设置相对路径
- `addSignature` -> 添加签名记录

**Phase 3: 构建签名文件**
- `getSignAlgOID` -> 获取签名算法 OID
- `getSeal` -> 获取印章
- `getAppearance` -> 获取签章外观
- `toBeDigestFileList` -> 构造待摘要文件列表
- `calculateFileDigest` -> 计算文件摘要
- `flushFileByName` -> 刷新签名文件

**Phase 4: 计算签名值**
- `signContainer.sign` -> 签名容器签名
- `Files.write` -> 写入签名值

**5 种签名容器**:

| 容器 | 说明 | easyofd-rust 当前 |
|:---|:---|:---:|
| GBT35275DSContainer | GB/T 35275 数字签名 | 部分 (SealInfo 简化版) |
| SESV1Container | SES V1 签章容器 | 30% |
| SESV4Container | SES V4 签章容器 | 0% |
| SESV5Container | SES V5 签章容器 | 0% |
| DigitalSignContainer | 数字签名容器 | 0% |

**3 种签章位置**:

| 位置 | 说明 | easyofd-rust 当前 |
|:---|:---|:---:|
| NormalStampPos | 普通签章 (单页矩形区域) | 已支持 (ElectronicSeal.position) |
| RidingStampPos | 骑缝章 (跨页分割) | 0% |
| CuttingRideStampPos | 裁切骑缝章 | 0% |

### 4.4 方向 D: ofdrw-gm SES V1/V4/V5 完整化

**SES V1 完整结构** (ASN.1 SEQUENCE):

```asn1
SESeal ::= SEQUENCE {
    header      SEQUENCE {
        id      OBJECT IDENTIFIER,
        version INTEGER,
        vid     UTF8String
    },
    esID        UTF8String,
    property    SEQUENCE {
        name    UTF8String,
        type    INTEGER,      -- 0=公章, 1=私章
        ...
    },
    picture     OCTET STRING, -- 印章图片
    extDatas    SEQUENCE OF ExtData OPTIONAL,
    signInfo    SES-SignInfo
}

SES-SignInfo ::= SEQUENCE {
    cert        OCTET STRING, -- X.509 证书 DER
    sigAlgOID   OBJECT IDENTIFIER,
    signData    OCTET STRING  -- SM2 签名值
}
```

**当前 easyofd-rust SealInfo** (简化版，仅 3 字段):

```rust
pub struct SealInfo {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub cert_der: Vec<u8>,   // 对应 cert
    pub image: Vec<u8>,      // 对应 picture
    pub version: u32,        // 对应 header.version
    // 缺失: esID, property, extDatas, signInfo.sigAlgOID, signInfo.signData
}
```

**SES V4 差异** (vs V1):
- 展平 cert/alg/sig 到 SESeal 顶层 (不再嵌套 SES-SignInfo)
- CertList 支持 full cert OR digest
- 时间格式改为 GeneralizedTime

**SES V5 差异** (vs V4):
- 等同 V4 + 可选 timeStamp 字段

**实施优先级**: V1 先行 -> V4 -> V5 (V5 是 V4 的超集，实现成本递减)

---

## 5. 阶段化路线 Phased Roadmap

### 总览

| 阶段 | 名称 | 工作量 | 关键交付 | 覆盖率目标 | 状态 |
|:---:|:---|:---:|:---|:---:|:---:|
| P0 | 基准巩固 | 1 周 | 修复 ignored 测试 + CI + 文件拆分 | >= 92% | ✅ 完成 |
| P1 | 核心布局 | 4 周 | 确定性阅读顺序分析 | >= 90% | ✅ 完成 |
| P2 | 核心补全 | 4 周 | action/annotation/attachment/versions | >= 90% | ✅ 完成 |
| P3 | 签章深化 | 3 周 | SES V1 完整 + checkSealMatch + PKCS#12 | >= 90% | ✅ 完成 |
| P4 | 版面引擎 2 | 3 周 | LayoutAnalyzer 确定性分析 | >= 90% | ✅ 完成 |
| P5 | 签章守护 | 2 周 | V4/V5 + 完整验证 | >= 90% | ✅ 完成 |
| P6 | 加密 | 2 周 | SM4-CBC/ECB + archive integrity | >= 90% | ✅ 完成 |
| P7 | 易用性 | 2 周 | EasyOfd 门面 + 22 examples + 文档 | >= 85% | ✅ 完成 |
| P8 | 生产 | 2 周 | 6 CI workflows + 70 样本 + 比对 + v0.1.1 发布 | >= 90% | ✅ 完成 |

### 甘特图 (简化)

```
Week:  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24
P0:    |
P1:       |-----------------------|
P2:                               |-----------------------|
P3:                                                    |---------------|
P4:                                                                  |---------------|
P5:                                                                               |---------|
P6:                                                                                          |---------|
P7:                                                                                                    |---------|
P8:                                                                                                              |---------|
```

### 里程碑

| 里程碑 | 阶段 | 预计日期 | 标志 | 状态 |
|:---|:---|:---|:---|:---:|
| M0: 基线固化 | P0 完成 | +1 周 | 0 ignored, CI 全绿 | ✅ 已达成 |
| M1: 布局引擎 | P1 完成 | +5 周 | 确定性阅读顺序分析 | ✅ 已达成 |
| M2: 核心补全 | P2 完成 | +9 周 | action/annotation/attachment/versions 全部实现 | ✅ 已达成 |
| M3: 签章可用 | P3 完成 | +12 周 | SES V1/V4/V5 + checkSealMatch + PKCS#12 | ✅ 已达成 |
| M4: 渲染引擎 | P4 完成 | +15 周 | LayoutAnalyzer 确定性分析 | ✅ 已达成 |
| M5: 签章完整 | P5 完成 | +17 周 | V1/V4/V5 全覆盖 + SM2WithSM3 | ✅ 已达成 |
| M6: 加密可用 | P6 完成 | +19 周 | SM4-CBC/ECB + archive integrity | ✅ 已达成 |
| M7: 用户友好 | P7 完成 | +21 周 | 22 examples + 完整文档 | ✅ 已达成 |
| M8: v0.1.1 | P8 完成 | +24 周 | crates.io 发布 21/21 crate（2026-08-21） | ✅ 已达成 |

---

## 6. 双向精确比对验证策略 Verification Strategy

### 6.1 设计原则

- **不依赖 JDK**: 比对流程纯 Rust + 预生成产物
- **预生成 ofdrw 产物**: 用 Java ofdrw 对 70 个真实 OFD 样本生成预期 JSON/PDF，存入 `tests/expected/`
- **逐字段比对**: Rust 端对同样输入生成产物，与预期逐字段比对（70/70 零偏差已达成）
- **增量更新**: 每新增一个功能，先用 ofdrw 生成预期产物，再实现 Rust 端

### 6.2 比对维度

| 维度 | 比对方式 | 容差 |
|:---|:---|:---|
| XML 结构 | 语义比对 (忽略空白/属性顺序) | 精确 |
| ZIP entry 列表 | 集合比对 | 精确 |
| ZIP entry 内容 | 字节比对 | 精确 |
| 图片数据 | 哈希比对 (SHA-256) | 精确 |
| PDF 文本 | 文本提取后字符串比对 | 精略 (浮点精度) |
| PDF 坐标 | 数值比对 | 容差 0.01mm |
| 时间戳格式 | 格式化后字符串比对 | 容差 1 秒 |
| 签名值 | 字节比对 (相同输入应产生确定性签名) | 精确 (使用固定密钥) |

### 6.3 比对 Pipeline

```bash
# Step 1: P0 阶段 - 预生成 (一次性)
# 用 Java ofdrw 处理 tests/fixtures/real_ofd/*.ofd
# 输出到 tests/expected/
tests/expected/
├── simple_1.ofd.expected.json      # 结构化预期
├── simple_2.ofd.expected.json
├── signed.ofd.expected.json
├── with_table.ofd.expected.json
├── multi_page_image.ofd.expected.json
├── simple_1.ofd.expected.pdf       # PDF 预期 (用于转换比对)
└── ...

# Step 2: Rust 端生成 + 比对 (每次 CI)
# cargo test --test comparison
# 1. 读取 tests/fixtures/real_ofd/*.ofd
# 2. 用 Rust 端处理
# 3. 与 tests/expected/ 逐字段比对
```

### 6.4 预期产物 JSON 格式

```json
{
  "version": "1.0",
  "pages": [
    {
      "width": 210.0,
      "height": 297.0,
      "content_count": 3,
      "text_objects": [
        {
          "x": 20.0,
          "y": 30.0,
          "text": "Invoice #001",
          "font": "SimSun",
          "size": 18.0,
          "bold": true
        }
      ],
      "image_count": 1,
      "path_count": 0
    }
  ],
  "signatures": {
    "count": 1,
    "algorithm": "SM2WithSM3",
    "references_valid": true
  },
  "zip_entries": [
    "OFD.xml",
    "Doc_0/Document.xml",
    "Doc_0/Pages/Page_0/Content.xml"
  ]
}
```

---

## 7. 风险与不可行项 Risks

### 7.1 高风险

| 风险 | 影响 | 缓解措施 |
|:---|:---|:---|
| **layout XY-cut 复杂度** | XY-cut 算法在复杂排版 (表格嵌套/多栏) 下可能退化为 O(n^2) | P1 阶段先实现简单聚类排序，XY-cut 作为可选优化，复杂场景降级到顺序布局 |
| **双向比对的精度** | PDF 浮点精度差异 (f32 vs f64)、时间戳格式差异可能导致误报 | 设置合理容差 (坐标 0.01mm, 时间 1秒)，比对代码用 `approx` crate |
| **ofdrw-markdown 转换保真度** | 复杂 OFD (表格/多栏/浮动对象) 的 Markdown 转换必然有损失 | 明确文档化损失范围，提供 loss report，不追求 100% 保真 |

### 7.2 中风险

| 风险 | 影响 | 缓解措施 |
|:---|:---|:---|
| **SM4 加密依赖** | 纯 Rust SM4 实现可能不成熟 | 优先评估 `sm4` crate 质量，必要时使用 `openssl` FFI (违反 unsafe 禁令需特别审批) |
| **字体渲染** | OFD 字体嵌入/子集化复杂，当前仅支持注册 API | P7 阶段仅提供字体注册接口，实际字形渲染使用系统字体 fallback |
| **大文件性能** | 100MB+ OFD 文件的签名/加密可能超时 | 流式处理 + 进度回调 + 超时保护 |

### 7.3 已知不可行项

| 项目 | 原因 |
|:---|:---|
| 100% PDF 保真转换 | PDF 和 OFD 是不同标准，文本编码/字体/颜色空间存在本质差异 |
| 实时 OCR | 超出库的职责范围，应由调用方提供 OCR 结果 |
| 自定义 CJK 字体生成 | 字体设计/子集化是独立工程，建议使用现有字体资源 |
| 异步 I/O | OFD 操作是文件密集型，同步 I/O 足够，引入 async 运行时增加复杂度无收益 |

---

## 8. 待确认事项

以下事项需要用户确认后方可最终确定实施计划：

| # | 事项 | 选项 | 默认建议 |
|:---:|:---|:---|:---|
| 1 | **P1 XY-cut 是否纳入** | A) P1 纳入 / B) 延后到 P4 / C) 不做 | B (先聚类，后优化) |
| 2 | **P6 加密方案** | A) 纯 Rust SM4 / B) openssl FFI / C) 延后 | A (纯 Rust) |
| 3 | **双向比对的 ofdrw 预期产物** | A) 存仓库 (约 5MB) / B) CI 时动态生成 | A (确定性更强) |
| 4 | **P2 优先级调整** | action/annotation/attachment/versions 是否全部 P2？或拆分 | 保持 P2 全部 |
| 5 | **字体渲染范围** | A) 仅系统字体 fallback / B) 嵌入字体子集化 | A (P7 阶段) |
| 6 | **v1.0.0 发布时机** | A) P8 完成后发布 / B) P6 完成后发布 | A (功能完整) |
| 7 | **CI 平台** | A) 仅 GitHub Actions / B) 增加 GitLab CI | A (当前已有) |
| 8 | **测试样本来源** | A) 仅现有 5 个 / B) 从 ofdrw 仓库补充 / C) 自行生成 | ✅ 已采用 B：70 个 ofdrw 样本 |

---

## 附录 A: 术语表

| 术语 | 说明 |
|:---|:---|
| OFD | Open Fixed-layout Document, GB/T 33190-2016 国家标准 |
| SES | Secure Electronic Seal, 安全电子签章 |
| GB/T 38540 | 信息安全技术 电子签章密码技术规范 |
| GB/T 35275 | 信息安全技术 SM2 密码算法使用规范 |
| SM2 | 国密椭圆曲线公钥密码算法 |
| SM3 | 国密杂凑算法 (256 位摘要) |
| SM4 | 国密分组密码算法 (128 位密钥) |
| Div | CSS 盒式模型中的块级容器元素 |
| XY-cut | 基于 X/Y 轴投影的页面切分算法 |
| VPage | Virtual Page, 虚拟页面 (排版引擎的中间表示) |

## 附录 B: 参考资料

| 资料 | 链接/位置 |
|:---|:---|
| ofdrw v2.4.0 | https://github.com/ofdrw/ofdrw |
| GB/T 33190-2016 | 电子文件存储与交换格式 |
| GB/T 38540-2020 | 信息安全技术 电子签章密码技术规范 |
| easyofd-rust Architecture | docs/easyofd-rust-Architecture.zh_CN.md |
| easyofd-rust 技术选型 | docs/easyofd-rust-技术选型.md |
| easyofd-rust 生产就绪计划 | docs/easyofd-rust-生产就绪计划.md |
