# Phase 1: Route 1 全元素树迁移

> **阶段**: XmlElement trait + XmlNode 树 + 管线迁移
> **时间跨度**: 2026-08-10 ~ 2026-08-11
> **状态**: ✅ 已完成

## 目标

将 OFD XML 解析/生成从扁平字符串拼接迁移为结构化的 XmlNode 元素树，建立 `XmlElement` trait 作为所有 OFD 元素的统一序列化/反序列化接口，实现 reader/writer 管线全面迁移。

## 范围

- `easyofd-core`: XmlElement trait + XmlNode + quick-xml 解析桥接
- `easyofd-core`: 40+ 元素类实现 XmlElement（to_xml / from_xml + roundtrip 测试）
- `easyofd-reader`: 全部入口改用 XmlNode 树解析
- `easyofd-writer`: build_document_xml 改用 XmlNode 树生成

## 方案

### XmlElement trait 设计

```rust
pub trait XmlElement {
    fn tag_name(&self) -> &str;
    fn to_xml(&self, writer: &mut XmlWriter<impl Write>) -> Result<(), OfdError>;
    fn from_xml(element: &XmlElementRef) -> Result<Self, OfdError> where Self: Sized;
}
```

### XmlNode 树结构

```rust
pub struct XmlNode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<XmlNode>,
    pub text: Option<String>,
}
```

### 管线迁移策略

1. **试点阶段**: parse_document_entry 改用 XmlNode 树（roundtrip 60/60 保持）
2. **reader 侧**: 全部入口改用 XmlNode 树解析
3. **writer 侧**: 全部 build_* 改用 XmlNode 树生成
4. **验证**: 每步都运行 roundtrip_diff 确保 60/60 零偏离

## 任务列表

- [x] 设计 XmlElement trait（tag_name / to_xml / from_xml）
- [x] 实现 XmlNode 树结构 + quick-xml 解析桥接
- [x] 实现 40 个元素类的 XmlElement（CT_PageArea / CT_TextObject / CT_ImageObject 等）
- [x] 管线迁移试点：parse_document_entry 改用 XmlNode 树
- [x] reader 侧全面迁移：全部入口改用 XmlNode 树
- [x] writer 侧全面迁移：全部 build_* 改用 XmlNode 树
- [x] CT_PageArea XmlElement 修正：PhysicalBox 等改为子元素（对齐 ofdrw 输出）
- [x] roundtrip_diff 60/60 零偏离验证

## 验证标准

| 维度 | 标准 | 实际状态 |
|---|---|---|
| XmlElement 实现数 | ≥ 40 | ✅ 40+ 元素类 |
| roundtrip_diff | 60/60 零偏离 | ✅ 0 ZIP + 0 XML = 0 |
| 测试 | 全部通过 | ✅ 1164+ tests |
| 管线迁移 | reader + writer 全部改用 XmlNode | ✅ |

## 状态

**✅ 已完成** — commits `3133073` ~ `86d8a6e`

## 证据

- `crates/easyofd-core/src/xml_element.rs`: XmlElement trait 定义（276 行）
- `crates/easyofd-core/src/xml_impls.rs`: 27 个 XmlElement 实现
- `crates/easyofd-core/src/xml_parse.rs`: XmlNode 解析桥接（166 行）
- 15 个文件实现 XmlElement trait（page_description / graph / 等）
- `crates/easyofd/tests/roundtrip_diff.rs`: 60 样本零偏离验证
- commit `86d8a6e`: writer 侧完成（roundtrip 60/60 保持）
- commit `c306e6b`: reader 侧完成
