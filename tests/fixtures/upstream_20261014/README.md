# 固定OFDRW增量文字样本

参考提交 `84e9eb260ba6fd351ecccf08a01a40305d4af635`，版本2.4.0。
`tools/upstream/TextFixtures.java` 移植上游两个测试类的输入构造，不把输入生成等同于Java输出验证。

- 描边矩阵：false（显式色）、省略（继承色）、true（显式/继承色），以及Fill=false的仅描边。
- 固定输入字号20mm；线宽改为0.15mm，使黑填充和红描边都可观察。上游原始1mm描边可能完全覆盖细字形的填充。
- AB使用1000mm页面以保留上游坐标断言；竖排采用3.175mm字号、3.18mm步距和非零CTM平移。
- 嵌入字体来自上游公开测试资源，来源和哈希见manifest；仅按上游仓库许可证记录来源，未独立裁定字体再分发许可。
- 完整Java源码构建、真实上游测试与实际输出对比另见验证报告；普通Rust测试直接使用样本，无需JDK。

重建：先设置`OFDRW_SOURCE_DIR`为固定提交的现有检出，再运行`bash scripts/generate_upstream_text_fixtures.sh`。脚本校验SHA/字体和生成结果，不pull/reset外部仓库。
