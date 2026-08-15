# easyofd-rust vs ofdrw (Java) 性能对比基线报告

> 基线日期: 2026-08-16
> 首次建立，供 CI 回归对比

---

## 1. 方法学

### 1.1 机器环境

| 项目 | 值 |
|---|---|
| 架构 | arm64 (Apple Silicon) |
| OS | macOS 26.5.2 (Darwin 25F84) |
| Rust toolchain | 1.97.1 (2026-07-14) |
| Rust edition | 2024 |
| Java | OpenJDK 21.0.12 (Microsoft LTS) |
| ofdrw 版本 | 2.4.0 (Maven Central) |

### 1.2 统计口径

| 侧 | 工具 | 预热 | 测量轮次 | 指标 |
|---|---|---|---|---|
| Rust | Criterion 0.8 | 3s 自适应 | 100 samples | 中位数 (p50) ± 置信区间 |
| Java | 纯 `System.nanoTime()` | 10 轮 | 50 轮 | 中位数 (p50) |

### 1.3 场景与路径说明

| 场景 | Rust 侧路径 | Java 侧路径 |
|---|---|---|
| **写 (text)** | `OfdWriter` 模型直拼：`OfdPage::add_text()` → `build()` 生成 ZIP 字节 | `OFDDoc` + `Paragraph` 流式布局引擎：含分页、段落排版 → flush → 写文件 |
| **写 (image)** | `OfdWriter` + `ImageObject::jpeg()` → `build()` | `OFDDoc` + `Paragraph` + `Img` → flush → 写文件 |
| **读 (text)** | `OfdReader::from_bytes()` 内存解析 + `extract_all_text()` | `OFDReader(Path)` 解压到磁盘 + XML 解析 + 遍历页面 |
| **读 (image)** | 同上 | 同上 |
| **roundtrip (text)** | 读 (from_bytes) → 写 (OfdWriter::build) | 写文件 → OFDReader 解析 → 再写文件 |
| **roundtrip (image)** | 同上 | 同上 |

**关键路径差异**:

1. **写入路径**: Java 使用 `OFDDoc` 含完整布局引擎（自动分页、段落排版、浮动布局），Rust 使用 `OfdWriter` 纯模型直拼（无布局引擎）。因此 Java 写入耗时包含布局引擎开销。
2. **读取路径**: Java `OFDReader(Path)` 需要先解压 OFD 到临时目录再解析 XML，Rust `OfdReader::from_bytes()` 直接在内存中解压+解析。Java 包含文件 I/O 开销。
3. **Roundtrip**: Java 包含两次文件写入 + 一次磁盘读取，Rust 全程内存操作。

### 1.4 样本矩阵

每页 30 行文字（纯文本场景）或 30 行文字 + 1 张 JPEG 图片（图片场景）。

| 页数 | 纯文本样本大小 (Rust) | 图片样本大小 (Rust) |
|---|---|---|
| 1 | ~3 KB | ~4 KB |
| 10 | ~28 KB | ~37 KB |
| 100 | ~275 KB | ~362 KB |

### 1.5 JVM 参数

```
-Djava.awt.headless=true -Xms256m -Xmx512m
```

### 1.6 Criterion 输出目录

Rust criterion 输出位于 `target/criterion/`，子目录按 benchmark group 命名：
- `read_text_30_lines/` — 读取纯文本
- `read_text_plus_image/` — 读取文本+图片
- `write_text_30_lines/` — 写入纯文本
- `write_text_plus_image/` — 写入文本+图片
- `roundtrip_text_30_lines/` — roundtrip 纯文本（reader 侧）
- `roundtrip_text_plus_image/` — roundtrip 图片（reader 侧）
- `roundtrip_write_read_write_text/` — roundtrip 纯文本（writer 侧）
- `roundtrip_write_read_write_image/` — roundtrip 图片（writer 侧）

CI 回归对比：`cargo bench` 输出 HTML 报告到 `target/criterion/report/index.html`。

---

## 2. 结果

### 2.1 写入性能 (Write)

| 场景 | 页数 | Rust (ms) | Java (ms) | Java / Rust |
|---|---|---|---|---|
| write_text | 1 | 0.111 | 4.675 | 42.1x |
| write_text | 10 | 0.746 | 13.204 | 17.7x |
| write_text | 100 | 7.286 | 111.534 | 15.3x |
| write_image | 1 | 0.135 | 2.634 | 19.5x |
| write_image | 10 | 0.849 | 8.201 | 9.7x |
| write_image | 100 | 7.890 | 89.085 | 11.3x |

### 2.2 读取性能 (Read)

| 场景 | 页数 | Rust (ms) | Java (ms) | Java / Rust |
|---|---|---|---|---|
| read_text | 1 | 0.066 | 3.905 | 59.2x |
| read_text | 10 | 0.525 | 9.223 | 17.6x |
| read_text | 100 | 5.204 | 57.181 | 11.0x |
| read_image | 1 | 0.074 | 1.986 | 26.8x |
| read_image | 10 | 0.560 | 5.879 | 10.5x |
| read_image | 100 | 5.456 | 37.678 | 6.9x |

### 2.3 Roundtrip 性能 (Read → Write)

| 场景 | 页数 | Rust (ms) | Java (ms) | Java / Rust |
|---|---|---|---|---|
| roundtrip_text | 1 | 0.174 | 8.886 | 51.1x |
| roundtrip_text | 10 | 1.286 | 28.328 | 22.0x |
| roundtrip_text | 100 | 12.373 | 273.780 | 22.1x |
| roundtrip_image | 1 | 0.205 | 6.588 | 32.1x |
| roundtrip_image | 10 | 1.427 | 19.851 | 13.9x |
| roundtrip_image | 100 | 13.285 | 211.413 | 15.9x |

---

## 3. 解读

### 3.1 Rust 优势来源（架构层面）

1. **零成本内存操作**: Rust 的 `OfdReader::from_bytes()` 和 `OfdWriter::build()` 全程在内存中完成，无文件 I/O 系统调用开销。Java 的 `OFDReader(Path)` 需要先将 OFD 解压到临时目录（涉及文件系统创建、写入、读取），这部分 I/O 开销在小文档场景（1 页）尤为显著，解释了为什么 1 页时差距最大（50-60x）。

2. **无布局引擎开销**: Rust 的 `OfdWriter` 是纯模型直拼——直接将 `OfdPage` 中的 `TextObject`/`ImageObject` 序列化为 XML 并打包为 ZIP，不做任何排版计算。Java 的 `OFDDoc` 包含完整的流式布局引擎（段落分页、浮动布局、字体度量），即使只添加简单 `Paragraph` 也会触发布局管线。这是写入场景差距的主要来源。

3. **JIT 预热摊平**: 随着文档规模增大（100 页），JVM JIT 编译效果显现，Java/Rust 比值从 1 页的 40-60x 下降到 100 页的 7-22x。100 页时布局引擎和 XML 解析的固定开销被摊平，差距主要来自每次操作的边际成本差异。

### 3.2 差距可优化方向

1. **Java 读取侧**: 如果使用 `OFDReader` 的内存模式（而非 Path 解压），可以消除文件 I/O 开销。但 ofdrw 的 `OFDReader` API 设计以 Path 为主，内存模式支持有限。

2. **Java 写入侧**: 如果绕过 `OFDDoc` 布局引擎，直接操作 `OFDDir` + `DocDir` 底层 API 进行模型直拼（类似 Rust 的 `OfdWriter`），写入性能差距可大幅缩小。但这需要手写 XML 序列化和 ZIP 打包逻辑。

3. **Rust 读取侧**: 当前 `extract_all_text()` 遍历所有页面并拼接字符串，100 页时字符串分配成为瓶颈。如果只需要结构化解析（不需要文本提取），读取速度可以更快。

### 3.3 不可比因素（需在对比中注明）

- **功能完整性**: Java 的 `OFDDoc` 支持自动分页、浮动布局、字体嵌入等完整排版功能，Rust 的 `OfdWriter` 仅做模型直拼。两者不在同一功能层级。
- **文件 I/O**: Java 读取涉及磁盘解压，Rust 读取是纯内存操作。对比时应注明此差异。
- **GC 压力**: Java 在 100 页 roundtrip（273ms）时可能受 GC 暂停影响，Rust 无此问题。

---

## 4. 基线快照

| 文件 | 说明 |
|---|---|
| `target/criterion/` | Criterion 完整输出（HTML 报告 + 统计数据） |
| `target/criterion/report/index.html` | 可视化对比报告 |
| `crates/easyofd-reader/benches/read_bench.rs` | Rust 读取 + roundtrip 基准源码 |
| `crates/easyofd-writer/benches/write_bench.rs` | Rust 写入 + roundtrip 基准源码 |
| `.ofdrw-gen/PerfBenchmark.java` | Java 基准源码 |

**CI 回归对比方法**: 运行 `cargo bench --workspace`，criterion 自动与上次基线对比，检测性能回归（>5% 标记为变化）。
