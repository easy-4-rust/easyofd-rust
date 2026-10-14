# 无豁免依赖治理
状态：IMPLEMENTING。基线dev@44e6d8a。用户授权在当前dev实施、提交并推送，不建分支、不发布。
目标：消除RUSTSEC-2026-0187、RUSTSEC-2026-0192、RUSTSEC-2024-0370；保留全部已有功能及Rust1.88兼容。
事实源：本规格及已有生产验收支持范围；Superpowers用于执行，不引入第二套体系。
1. PDF生成依赖升级printpdf0.12.8，嵌入字体、Unicode文本、显式glyph、文字坐标/CTM/描边、图片尺寸、路径、分页及公共入口保持契约。新依赖树不得含旧lopdf或proc-macro-error/error2。
2. 字体解析迁移skrifa（初选0.44，MSRV1.85）；SVG/PNG轮廓、CJK探测与度量保持，替换fontdue栅格化，保留字体回退及资源预算。不得通过关闭PDF/PNG/字体功能消除依赖。
3. 根与Fuzz锁文件不能含ttf-parser、fontdue、proc-macro-error、proc-macro-error2或lopdf<0.42。删除全部审计豁免，配置审计及独立无配置审计不得有漏洞/停止维护/撤回项。
4. 保持Rust1.88三平台六矩阵及stable；完整测试、覆盖率≥90%、clippy/fmt、四Fuzz各100000、2GiB/30s真实转换烟测通过。
5. 独立审查修复Important/Critical。新SHA CI完成后才归档；不扩大既有生产支持范围，不宣称部署。
