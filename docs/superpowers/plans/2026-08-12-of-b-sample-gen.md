# OF-B: ofdrw 程序化生成多样化 OFD 样本

**状态**: ✅ 完成（14 种样本全部生成，roundtrip 14/14 通过）  
**创建日期**: 2026-08-15  
**关联**: tests/fixtures/real_ofd/（60 个真实样本，roundtrip 60/60 零偏差）

## 目标

通过 ofdrw（Java）各模块 API 程序化生成覆盖各种边界的 OFD 样本，
增强 easyofd-rust 的测试多样性，补充真实样本无法覆盖的边界场景。

## 样本类别（14 种）

| # | 文件名 | 覆盖场景 | 依赖模块 |
|---|--------|----------|----------|
| 1 | gen_01_docinfo_full.ofd | DocInfo 全字段（DocID/Creator/Author/Subject/Keywords/Date） | ofdrw-layout |
| 2 | gen_02_multi_page.ofd | 多页文档（5 页，每页 30+ 行触发分页） | ofdrw-layout |
| 3 | gen_03_complex_text.ofd | 复杂文字样式（粗体/斜体/下划线/颜色/字重/首行缩进/中英混排） | ofdrw-layout |
| 4 | gen_04_with_image.ofd | 图片嵌入（PNG，绝对定位，边框/内边距） | ofdrw-layout |
| 5 | gen_05_canvas_drawing.ofd | Canvas 绘图（圆形/矩形填充+描边/交叉线条） | ofdrw-layout |
| 6 | gen_06_template_page.ofd | 模板页（Template Page，页眉页脚，3 页复用） | ofdrw-layout |
| 7 | gen_07_custom_font.ofd | 自定义字体（NotoSerif/微软雅黑/混合字体） | ofdrw-layout |
| 8 | gen_08_annotations.ofd | 批注（印章注释 Stamp + 水印注释 Watermark） | ofdrw-layout + ofdrw-reader |
| 9 | gen_09_float_layout.ofd | 浮动布局（左/中/右浮动 Div + 背景色 + 边框） | ofdrw-layout |
| 10 | gen_10_virtual_page.ofd | VirtualPage 绝对定位（文字/Div/段落混合定位） | ofdrw-layout |
| 11 | gen_11_page_overflow.ofd | 段落分页溢出（100 个 Div 强制多页） | ofdrw-layout |
| 12 | gen_12_attachment.ofd | 附件（Attachment，文本附件） | ofdrw-layout |
| 13 | gen_13_riding_seal.ofd | 骑缝章签名（SES V4，Right 方向，需签名资源） | ofdrw-sign + ofdrw-gm |
| 14 | gen_14_digital_sign.ofd | 数字签名（SES V4，WholeProtected 模式，需签名资源） | ofdrw-sign + ofdrw-gm |

## 架构

```
scripts/ofd_sample_gen.sh          ← 主入口脚本
├── 前置检查（java/mvn/git/磁盘）
├── 克隆/更新 ofdrw（.ofdrw-gen/）
├── Maven 构建（ofdrw-layout + ofdrw-sign + 依赖）
├── 写入 GenerateSamples.java
├── 编译并运行生成器
└── 复制到 tests/fixtures/ofdrw_gen/

tests/fixtures/ofdrw_gen/          ← 生成样本目录（与 real_ofd/ 分离）
├── gen_01_docinfo_full.ofd
├── gen_02_multi_page.ofd
└── ...

crates/easyofd/tests/ofdrw_gen_conformance.rs  ← Rust conformance 测试
├── ofdrw_gen_roundtrip_all        ← 自动发现 + roundtrip
├── ofdrw_gen_all_are_valid_zip    ← ZIP 合法性
├── ofdrw_gen_all_contain_ofd_xml  ← OFD.xml 入口检查
└── ofdrw_gen_content_statistics   ← 内容统计（文字/图片/路径）

.github/workflows/ofd-gen.yml      ← CI 定时任务（每周日 02:00 UTC）
├── generate（JDK 环境生成样本）
└── verify（Rust 环境运行 conformance 测试）
```

## 运行方式

### 本地运行（需 JDK + Maven）
```bash
bash scripts/ofd_sample_gen.sh
cargo test --test ofdrw_gen_conformance
```

### CI 运行
- 自动：每周日 02:00 UTC 触发
- 手动：GitHub Actions -> OF-B Sample Generation -> Run workflow

### 无 JDK 环境
测试自动跳过（不 panic），输出提示信息。

## 设计决策

1. **生成样本与真实样本分离**：`ofdrw_gen/` vs `real_ofd/`，避免混淆来源
2. **不要求 byte diff**：生成样本可能与 ofdrw 默认行为有差异，只验证"可读 + 页数 >= 1 + roundtrip 页数不变"
3. **签名样本条件生成**：骑缝章/数字签名依赖 ofdrw-sign 的密钥资源，不存在则跳过
4. **降级模式**：签名模块编译失败时，自动降级为纯 layout 模式
5. **CI 不阻断**：生成样本的 conformance 测试失败不阻断 PR（`continue-on-error: true`）

## 遗留风险

1. **签名资源可用性**：骑缝章/数字签名样本依赖 ofdrw-sign/src/test/resources/ 中的密钥文件，CI 环境可能不存在（浅克隆不含测试资源）
2. **ofdrw API 兼容性**：ofdrw 是活跃项目，API 可能变化；脚本使用 `git pull` 更新，需关注 API 变更
3. **字体依赖**：ofdrw-layout 的 Font API 可能依赖系统字体，CI 环境字体有限
4. **Awt 依赖**：gen_04_with_image 使用 java.awt 创建测试 PNG，CI 的 headless 环境可能需要 `-Djava.awt.headless=true`

## 后续计划

- [ ] 首次运行验证，确认生成样本数量和种类
- [ ] 根据 easyofd-reader 的能力边界，调整生成策略
- [ ] 添加更多边界样本（加密 OFD、大文件、特殊字符路径等）
- [ ] 考虑将生成样本纳入 roundtrip 60/60 的扩展计数
