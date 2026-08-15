# 性能对比基线建立计划

> 日期: 2026-08-16
> 状态: ✅ 已完成

## 目标

建立 easyofd-rust 与 ofdrw (Java) 的性能对比基线，覆盖读/写/roundtrip 三个场景，样本矩阵为 1/10/100 页 × 纯文本 + 文本+图片。

## 方法

### Rust 侧
- 使用 Criterion 0.8 基准框架
- 扩展 `crates/easyofd-reader/benches/read_bench.rs`：新增 read_text、read_image、roundtrip_text、roundtrip_image 四组 benchmark
- 扩展 `crates/easyofd-writer/benches/write_bench.rs`：新增 write_text、write_image、roundtrip_write_read_write_text、roundtrip_write_read_write_image 四组 benchmark
- 每组 3 个样本点（1/10/100 页），共 24 个 benchmark

### Java 侧
- 编写 `.ofdrw-gen/PerfBenchmark.java`：纯计时基准（不引入 JMH）
- 预热 10 轮 + 测量 50 轮取中位数
- 使用 JDK 21，`-Djava.awt.headless=true -Xms256m -Xmx512m`
- 写入路径使用 `OFDDoc` + `Paragraph`（含布局引擎）
- 读取路径使用 `OFDReader(Path)`（解压+解析）

### 路径差异（已明确标注）
- **写入**: Java 使用布局引擎（OFDDoc），Rust 使用模型直拼（OfdWriter）——不在同一功能层级
- **读取**: Java 需解压到磁盘再解析，Rust 内存直接解析
- **Roundtrip**: Java 含文件 I/O，Rust 纯内存

## 结果摘要

| 场景 | 1 页 Java/Rust | 10 页 Java/Rust | 100 页 Java/Rust |
|---|---|---|---|
| write_text | 42x | 18x | 15x |
| read_text | 59x | 18x | 11x |
| roundtrip_text | 51x | 22x | 22x |
| write_image | 20x | 10x | 11x |
| read_image | 27x | 11x | 7x |
| roundtrip_image | 32x | 14x | 16x |

详细数据和解读见 `docs/benchmark-report.md`。

## 产出物

| 文件 | 说明 |
|---|---|
| `docs/benchmark-report.md` | 完整对比报告（方法学 + 结果表 + 解读） |
| `crates/easyofd-reader/benches/read_bench.rs` | Rust 读取基准（扩展版） |
| `crates/easyofd-writer/benches/write_bench.rs` | Rust 写入基准（扩展版） |
| `.ofdrw-gen/PerfBenchmark.java` | Java 基准程序 |
| `target/criterion/` | Criterion 输出（HTML 报告 + 统计数据） |

## 验证命令

```bash
# Rust 基准
cargo bench --package easyofd-reader --package easyofd-writer 2>&1 | tail -20

# Java 基准
cd .ofdrw-gen && javac -cp "$CP" PerfBenchmark.java && java -Djava.awt.headless=true -Xms256m -Xmx512m -cp "$CP:." PerfBenchmark

# Clippy
cargo clippy --package easyofd-reader --package easyofd-writer --all-targets -- -D warnings

# Fmt
cargo fmt --all -- --check
```

## 状态

- [x] Rust 基准编写 + 运行
- [x] Java 基准编写 + 运行
- [x] 结果收集 + 报告
- [x] Clippy 门禁通过
- [x] Fmt 门禁通过
