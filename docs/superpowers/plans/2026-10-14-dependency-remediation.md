# 无豁免依赖治理 Implementation Plan
> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development.
**Goal:** 清除三项依赖风险且保持转换行为和Rust1.88。
**Architecture:** Task1升级PDF引擎并适配操作流，Task2将字体统一到Fontations及tiny-skia，Task3严格依赖门禁/无豁免审计和全平台验证。
**Tech Stack:** printpdf0.12.8, lopdf≥0.42, skrifa0.44, tiny-skia0.12。
**Spec:** ../specs/2026-10-14-dependency-remediation.md
## Global Constraints
- 在当前dev；公共API兼容；Rust1.88；覆盖率≥90%；所有字体/PDF/PNG功能保留。
- 目标问题包从根/Fuzz锁文件移除，不添加审计例外，不更换为proc-macro-error2。
## Review Focus
- CJK/TTC/OTF与嵌入字体缺字行为保持，字体子集含显式glyph。
- PDF矩阵符号、文字字号单位、图片尺度、透明度及多页正确。
- 空白字形与坏字体返回明确结果，避免静默丢失文本。
- 超大字形/异常坐标仍先校验再分配，保留2GiB/30s隔离契约。
- Fresh resolution及Fuzz不能重新引入问题包，Rust1.88不得提高。

### Task 1: PDF引擎升级
**Files:** Cargo.toml/Cargo.lock（控制器）；crates/easyofd-convert/src/lib.rs、pdf_text_renderer.rs及PDF测试（执行者）。
**Interfaces:** 保持ofd_to_pdf及转换公共入口；内部采用新版printpdf文档/操作流，Task2后续迁移pdf_text_renderer字体解析。
- [ ] Step 1: 依赖门禁/无配置审计复现RED，保存现有PDF正例行为。
- [ ] Step 2: printpdf0.12.8及操作API适配，PDF现有回归与新增行为测试GREEN。
- [ ] Step 3: 任务独立审查，保留功能，提交准确范围。

### Task 2: 字体链迁移
**Files:** font/font_face.rs、glyph_outline.rs、cjk_font.rs、svg_text_renderer.rs、pdf_text_renderer.rs字体调用、converter/raster.rs；根/convert依赖（控制器）。
**Interfaces:** FontFace<'a>::parse、units_per_em、number_of_glyphs、glyph_index、glyph_hor_advance及GlyphOutline::load；均使用Fontations真实解析/轮廓，不造占位。
- [ ] Step 1: 固定TTF/TTC/CJK字形、推进量、空白/坏字体、PNG阳性像素与预算的回归；依赖门禁保持RED。
- [ ] Step 2: skrifa替代ttf-parser，tiny-skia替代fontdue，受影响测试GREEN。
- [ ] Step 3: 任务独立审查并提交。

### Task 3: 无豁免与交付
**Files:** scripts/check_dependency_contract.py、deny.toml、.cargo/audit.toml、.github/workflows/security.yml、Fuzz lock、验收文档。
**Interfaces:** 根/Fuzz锁文件强制拒绝目标问题包，audit warnings拒绝；不降低许可证、安全或覆盖门禁。
- [ ] Step 1: 删除全部例外，无配置audit及deny、严格依赖门禁GREEN。
- [ ] Step 2: 完整workspace/tests/doc/clippy/fmt/cov、Rust1.88、四Fuzz及实际烟测，独立全变更审查。
- [ ] Step 3: 提交推送代码触发三平台/覆盖/安全，dispatch Fuzz；当前SHA全部通过再文档归档推送。
