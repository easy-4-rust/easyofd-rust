//! 自定义扩展数据序列。
//!
//! 对应 Java: `org.ofdrw.gm.ses.v1.ExtensionDatas` / `org.ofdrw.gm.ses.v5.ExtensionDatas`
//!
//! ASN.1 结构：
//! ```asn1
//! ExtensionDatas ::= SEQUENCE OF ExtData
//! ```
//!
//! 在 SES 结构中通过 `[0] EXPLICIT` 标签包裹：
//! ```asn1
//! extDatas  [0] EXPLICIT SEQUENCE OF ExtData OPTIONAL
//! ```

use super::ext_data::ExtData;
use super::{DerResult, encode_sequence};

/// 自定义扩展数据序列。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.ExtensionDatas` / `org.ofdrw.gm.ses.v5.ExtensionDatas`
///
/// 包含零或多个 [`ExtData`]，用于在印章/签章结构中携带厂商自定义扩展。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtensionDatas {
    /// 扩展数据列表。
    items: Vec<ExtData>,
}

impl ExtensionDatas {
    /// 创建空的扩展数据序列。
    #[must_use]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 从已有列表创建。
    #[must_use]
    pub fn from_items(items: Vec<ExtData>) -> Self {
        Self { items }
    }

    /// 追加一条扩展数据。
    pub fn push(&mut self, item: ExtData) {
        self.items.push(item);
    }

    /// 获取扩展数据列表引用。
    #[must_use]
    pub fn items(&self) -> &[ExtData] {
        &self.items
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 扩展数据数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// DER 编码（不含 `[0]` 外层标签，仅 `SEQUENCE OF ExtData`）。
    pub fn encode_der(&self) -> Vec<u8> {
        let mut inner = Vec::new();
        for item in &self.items {
            inner.extend_from_slice(&item.encode_der());
        }
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        out
    }

    /// DER 解码（输入为 `SEQUENCE OF ExtData` 的内容字节，不含 SEQUENCE 头）。
    pub fn decode_der_content(content: &[u8]) -> DerResult<Self> {
        let mut items = Vec::new();
        let mut pos = 0;
        while pos < content.len() {
            let (tag, val, next) = super::decode_tlv(content, pos)?;
            if tag != super::TAG_SEQUENCE {
                return Err(super::DerError("期望 ExtData 为 SEQUENCE"));
            }
            let mut full = Vec::with_capacity(val.len() + 4);
            encode_sequence(&val, &mut full);
            items.push(ExtData::decode_der(&full)?);
            pos = next;
        }
        Ok(Self { items })
    }
}

impl From<Vec<ExtData>> for ExtensionDatas {
    fn from(items: Vec<ExtData>) -> Self {
        Self::from_items(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sm2_struct::oids::{SM3, parse_oid};

    fn sample_ext_data() -> ExtData {
        ExtData::non_critical(parse_oid(SM3), vec![0x01, 0x02])
    }

    #[test]
    fn empty_roundtrip() {
        let eds = ExtensionDatas::new();
        assert!(eds.is_empty());
        assert_eq!(eds.len(), 0);
        let der = eds.encode_der();
        let decoded = ExtensionDatas::decode_der_content(&der[2..]).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn single_item_roundtrip() {
        let mut eds = ExtensionDatas::new();
        eds.push(sample_ext_data());
        let der = eds.encode_der();
        // 解码 SEQUENCE 内部
        let (seq_val, _) = super::super::decode_sequence(&der, 0).unwrap();
        let decoded = ExtensionDatas::decode_der_content(&seq_val).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded.items()[0], sample_ext_data());
    }

    #[test]
    fn multiple_items_roundtrip() {
        let mut eds = ExtensionDatas::new();
        eds.push(sample_ext_data());
        eds.push(ExtData::new(parse_oid(SM3), true, vec![0xAB]));
        let der = eds.encode_der();
        let (seq_val, _) = super::super::decode_sequence(&der, 0).unwrap();
        let decoded = ExtensionDatas::decode_der_content(&seq_val).unwrap();
        assert_eq!(decoded.len(), 2);
        assert!(decoded.items()[1].critical);
    }

    #[test]
    fn from_vec_conversion() {
        let items = vec![sample_ext_data()];
        let eds: ExtensionDatas = items.into();
        assert_eq!(eds.len(), 1);
    }
}
