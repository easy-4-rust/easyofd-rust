# OFDRW 文字渲染补齐 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将上游 #411/#412 的文字描边与 CTM 字距语义贯通到 Rust 的真实转换输出。

**Architecture:** 保留现有 TextObject/OfdPage 公共结构，以 OfdReader 私有旁路数据保存转换所需 TextRenderRun，新增只读获取接口。转换器共享逐字定位，PDF/SVG/PNG 分别消费其结果；原始 roundtrip 路径维持。

**Tech Stack:** Rust 1.88，edition 2024，已有 XML/CT_Text/TextCode、printpdf、fontdue、tiny-skia、lopdf；Java OFDRW 2.4.0@84e9eb26。

**Spec:** [2026-10-14-ofdrw-upstream-alignment.md](../specs/2026-10-14-ofdrw-upstream-alignment.md)。

状态：ARCHIVED；任务1–6全部完成。已授权当前dev实施与提交/推送；代码提交`65b21bf14ebaa55956addfbd9097f12a12872227`的六矩阵CI、格式与bench通过。

## Global Constraints

- 遵守关联规格全部约束；中文文档，snake_case 文件，每对象独立文件，mod.rs/lib.rs 仅声明/导出新增对象。
- 不破坏 TextObject/OfdPage struct literal、pages()、现有公开构造器、原始 ZIP/XML 回写。
- 字距/点位置计算用 mm；只在后端转换单位。不把 Java 旋转前中间坐标当最终页面坐标。
- 已有 CTM 数学函数与模型直接复用，不重复造类型；不得通过只测“不描边”冒充完整描边支持。
- 每个实现任务执行测试红→最小实现→绿→受影响回归；记录红灯实际原因。提交另需授权，不把提交作为本次任务要求。

## Review Focus

1. 多个 TextCode 省略 X/Y、压缩 delta 的 g 重复语法：按 OFD 续接语义，无丢字/错误计数（任务1/2）。
2. DrawParam 继承链、显式色覆盖、循环及缺失引用：确定优先级，循环/坏引用返回明确错误（任务1）。
3. Fill=false/Stroke=true 以及省略 Stroke：分别描边而不填充、默认不描边（任务4/5）。
4. 字体缺失与 CGTransform glyph 数量/索引异常：可解释错误或已文档化降级，不能静默漏字（任务3/5）。
5. 非零 CTM 平移、旋转、非零 DeltaY 和零平移：字距无平移累积，旧横排行为保留（任务2/3）。

## 文件与接口分工

新增 reader `src/text_render_run.rs` 为绘制元数据对象；`src/parser.rs` 提取，`src/ofd_reader.rs` 私有保存，`src/lib.rs` 仅导出。新增 convert `src/point/glyph_position.rs` 为输出位置对象，`src/utils/text_layout.rs` 为定位函数。后端修改既有 lib.rs/PDF、svg_exporter.rs 和 raster.rs；不重构无关内容。

TextRenderRun 约定字段：`content_index: usize`、`text: CT_Text`、`ctm: Option<[f64;6]>`、`line_width_mm: f64`。颜色继承解析后的有效值进入 CT_Text；保留 text_codes/cg_transforms。OfdReader 新增 `text_render_runs(&self, page_index: usize) -> OfdResult<&[TextRenderRun]>`，与 pages()/content 的顺序对应，越界返回错误。新类型为 Rust 转换元数据，不虚构 Java 对象来源。

GlyphPosition 约定字段：`character: char`、`glyph_id: Option<u32>`、`origin_mm: [f64;2]`、`linear_ctm: [f64;4]`；全部代表最终 OFD 页面空间。`layout_text_run(run: &TextRenderRun, advances_mm: &[f64]) -> OfdResult<Vec<GlyphPosition>>` 消费字体测量的备用步距；显式 delta 优先，缺失步距才使用 advances，长度/非有限数不合法返回错误。

### Task 1：保留转换文字元数据（R1/R5）

**Files:** Create `crates/easyofd-reader/src/text_render_run.rs`、`crates/easyofd-reader/tests/text_render_metadata.rs`；Modify reader 的 parser.rs/ofd_reader.rs/lib.rs。

**Interfaces:** Consumes 现有 CT_Text/TextCode/CT_CGTransform、资源解析；Produces 上述 TextRenderRun 与 OfdReader::text_render_runs。

- [x] Step 1：编写 `text_render_metadata` 集成测试，以真实 ZIP/XML 断言 Stroke/Fill 默认值、四种描边组合、DrawParam 继承覆盖、CTM、两个独立 TextCode 坐标/步距与 CGTransform；追加循环/缺失资源错误测试和旧 pages()/struct literal 编译兼容测试。
- [x] Step 2：运行 `cargo test -p easyofd-reader --test text_render_metadata`，记录因新接口/行为缺失产生的红灯；环境故障不算红灯。
- [x] Step 3：实现旁路数据存储及读取接口；复用现有 delta/资源类型并处理 g 重复语法，不改变简化模型公共字段；测试越界和非有限输入拒绝。
- [x] Step 4：上述目标测试及 `cargo test -p easyofd-reader -p easyofd-core -p easyofd-writer` 通过；核对已有 raw XML/entries 的 roundtrip 没有变化。
- [x] Step 5：记录接口、红/绿输出、兼容性审查；只在本任务证据齐全后勾选，提交需另有授权。

### Task 2：统一文字原点与步距计算（R3/R5）

**Files:** Create `crates/easyofd-convert/src/point/glyph_position.rs`、`src/utils/text_layout.rs`、`tests/text_coordinate_alignment.rs`；Modify point/mod.rs、utils/mod.rs 与按需 Cargo.toml 测试依赖。

**Interfaces:** Consumes Task1 TextRenderRun、字体备用 advances_mm；Produces 上述 layout_text_run 与 GlyphPosition。

- [x] Step 1：编写 `text_coordinate_alignment`，AB 断言 `(112,226)/(120,241)` mm；竖排断言首字 `(208,29.5)` mm、相邻欧氏距离3.18 mm且方向与 CTM 一致；覆盖 e=f=0、DeltaY、多 TextCode 续接、g delta、异常长度/NaN。
- [x] Step 2：运行 `cargo test -p easyofd-convert --test text_coordinate_alignment`，记录缺失定位实现的红灯。
- [x] Step 3：实现 layout_text_run，复用 ctm_transform_point/ctm_transform_vector；坐标继承、Boundary、字形映射按上游处理；显式 delta 不混入 e/f，应用阅读/字符方向时避免重复旋转。
- [x] Step 4：目标测试通过，`cargo test -p easyofd-convert utils::` 通过；记录上游带 CGTransform 与不带重载的语义对应。
- [x] Step 5：审查单位、旋转和每个字符/字形数量，保存证据；不因纯数学测试通过而标记 PDF 验收通过。

### Task 3：实际 PDF 路径接入逐字定位（R4/R5）

**Files:** Modify `crates/easyofd-convert/src/lib.rs`、按需 `src/converter/itext_maker.rs`；Create `tests/pdf_text_alignment.rs`。

**Interfaces:** Consumes reader 元数据、layout_text_run 输出；Produces 保持签名不变的 ofd_to_pdf/ItextMaker::convert，内部将 origin_mm/linear_ctm 转 PDF text matrix。

- [x] Step 1：编写 `pdf_text_alignment`，经真实 OFD→PDF、lopdf 解码矩阵断言 AB 和竖排位置/步距、普通横排不回归、多个 TextCode/CGTransform 字形映射；缺失字体和坏 glyph 索引须明确错误/记录可验证降级。
- [x] Step 2：运行 `cargo test -p easyofd-convert --test pdf_text_alignment`，记录当前整体 use_text 路径的行为失败。
- [x] Step 3：在现有 PDF 文本渲染中消费定位结果、字体度量、glyph 映射和线性变换；保留无 XML 元数据的简化模型旧路径，避免重复Y翻转/旋转。
- [x] Step 4：目标测试及 `cargo test -p easyofd-convert` 通过；用固定字体渲染实际竖排PDF，保存输出图/坐标差分及无挤压证据。渲染工具缺失须保留该验收未完成。
- [x] Step 5：记录 PDF 结构测试和视觉结果各自边界，检查公开 API 未改；禁止以 %PDF 文件头或可读性代替位置验收。

### Task 4：SVG 文字描边（R2/R5）

**Files:** Modify `crates/easyofd-convert/src/exporter/svg_exporter.rs`；Create `tests/svg_text_stroke.rs`。

**Interfaces:** Consumes Task1 reader 元数据（定位消费Task2），保持 SvgExporter::convert 公共接口。

- [x] Step 1：编写 `svg_text_stroke`，四种描边组合断言显式/继承红色 stroke 的有无，黑色填充保留；另测 Fill=false/Stroke=true 输出 fill=none、线宽/转义和旧模型路径。
- [x] Step 2：运行 `cargo test -p easyofd-convert --test svg_text_stroke`，确认 Stroke=true 阳性用例在当前代码失败。
- [x] Step 3：输出有门控的 stroke、fill、stroke-width，使用保留位置/变换；显式颜色优先于继承颜色，默认 Stroke=false。
- [x] Step 4：目标测试及现有 SVG 测试通过；验证同一嵌入字体样本导出可见描边、关闭后填充保留，保存SVG和截图。
- [x] Step 5：保存8项矩阵中的SVG四项证据与额外Fill用例结果，审查普通文本行为兼容。

### Task 5：PNG 文字描边（R2/R5）

**Files:** Modify `crates/easyofd-convert/src/converter/raster.rs`、`src/converter/awt_maker.rs`；Create `tests/png_text_stroke.rs`；按需新增独立 glyph 轮廓辅助文件。

**Interfaces:** 保留现有 RasterRenderer 简化页面接口，增量新增内部 `render_page_with_text_runs(&self, page: &OfdPage, runs: &[TextRenderRun]) -> OfdResult<RgbaImage>`，AWTMaker 文件转换路径调用它。

- [x] Step 1：编写 `png_text_stroke`，固定嵌入字体四种组合断言红像素有无且黑填充保留；Fill=false/Stroke=true 无黑填充；同时测不同DPI/线宽、字体缺失的明确结果。
- [x] Step 2：运行 `cargo test -p easyofd-convert --test png_text_stroke`，确认当前只填充使 Stroke=true 用例失败。
- [x] Step 3：使用现有字体轮廓读取与 tiny-skia 实现真实 glyph 描边，门控 stroke/fill、使用有效色和mm线宽；通过增量内部接口消费文字位置，不用外接矩形冒充glyph轮廓。
- [x] Step 4：目标测试及 `cargo test -p easyofd-convert converter::` 通过；保存四种组合和额外Fill用例图片，像素阈值/抗锯齿容差固定并说明。
- [x] Step 5：核验AWTMaker文件路径确实使用新实现；原字体静默跳过行为涉及兼容时区分新严格转换路径与旧简化接口，记录审查结论。

### Task 6：固定上游基线与完整回归门禁（R5/R6）

**Files:** Create `tests/fixtures/upstream_20261014/manifest.json`、该目录 README.md 和 OFD 样本；Create `scripts/generate_upstream_text_fixtures.sh`；Modify `scripts/verify_with_ofdrw.sh`、按需 `.github/workflows/ofd-gen.yml`、`docs/bidirectional-verification.md`。

**Interfaces:** 生成脚本使用 `OFDRW_SOURCE_DIR` 指定已存在源码、`OFDRW_COMMIT` 默认为固定84e9eb26；检查SHA，不自动 pull/reset/安装；输出样本+来源/SHA256/字体许可与工具版本。既有verify脚本默认参数保持兼容，新增显式基线选择。

- [x] Step 1：编写脚本基线契约检查（受控Git夹具），验证SHA不符、字体缺失、生成失败均非零退出，外部目录不被修改；汇总描边8项与两种坐标场景及R5附加场景清单。
- [x] Step 2：运行契约检查，记录固定基线支持缺失的失败；确认该脚本与此前 .ofdrw-clone 默认路径不混淆。
- [x] Step 3：实现基线参数与生成流程，使用上游两测试类的构造逻辑和固定嵌入字体；仅在明确生成命令中产出样本，CI无JDK的普通测试仍可用已存夹具。
- [x] Step 4：执行 `cargo test --workspace`、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`，以及 `cargo test -p easyofd --test roundtrip_diff`；记录MSRV/三平台项目CI和新实际输出回归结果。已有stable Clippy问题单独定位，不能用旧绿记录覆盖。
- [x] Step 5：对照R1–R6检查全部证据、红绿链和公开兼容性，更新本计划与索引；任一输出/平台门禁缺失则保持待验证，不归档。

## 完成记录

实际证据见 [2026-10-14验证报告](../../ofdrw-upstream-alignment-verification.md)。本地workspace测试、fmt、clippy、roundtrip已执行；固定Java源码构建和9个上游真实测试方法、PNG/SVG输出对比、实际PDF输出均已有记录。任务6 Step4/5已完成：跨平台CI和MSRV1.88实际通过，不以本地Mac结果代替。

执行裁决：通过新增font_name/font_data旁路字段连接嵌入字体；文字元数据错误只在转换接口请求时报告，保持原始读取/roundtrip兼容；旧简化writer的pt字号路径保留；CGTransform当前支持1:1显式glyph映射，非1:1映射明确报错，复杂塑形不作完成声明。

最终门禁：[CI六矩阵全部通过](https://github.com/easy-4-rust/easyofd-rust/actions/runs/38510929976)，对应代码`65b21bf14ebaa55956addfbd9097f12a12872227`。CI中显式豁免Rust1.99新增的assert_is_empty断言显示风格建议，其他warning仍为门禁。归档不扩大复杂字体塑形/证书信任的范围。
