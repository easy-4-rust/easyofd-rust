//! SES V1 电子印章 ASN.1 结构定义。
//!
//! 对应 Java 版 [`org.ofdrw.gm.ses.v1`](https://github.com/ofdrw/ofdrw) 包，
//! 实现 GB/T 38540-2020 标准规定的 V1 版本 SESeal / SES_Signature 结构。
//!
//! # 结构概览
//!
//! ```asn1
//! SESeal ::= SEQUENCE {
//!     esealInfo  SES_SealInfo,
//!     signInfo   SES_SignInfo
//! }
//! SES_SealInfo ::= SEQUENCE {
//!     header     SES_Header,
//!     esID       IA5String,
//!     property   SES_ESPropertyInfo,
//!     picture    SES_ESPictureInfo,
//!     extDatas   [0] EXPLICIT SEQUENCE OF ExtData OPTIONAL
//! }
//! SES_Header ::= SEQUENCE {
//!     id         IA5String,  -- "ES"
//!     version    INTEGER,
//!     vid        IA5String
//! }
//! SES_ESPropertyInfo ::= SEQUENCE {
//!     type        INTEGER,
//!     name        PrintableString,
//!     certList    SEQUENCE OF OCTET STRING,
//!     createDate  UTCTime,
//!     validStart  UTCTime,
//!     validEnd    UTCTime
//! }
//! SES_ESPictureInfo ::= SEQUENCE {
//!     type     PrintableString,
//!     data     OCTET STRING,
//!     width    INTEGER,
//!     height   INTEGER
//! }
//! SES_SignInfo ::= SEQUENCE {
//!     cert               OCTET STRING,
//!     signatureAlgorithm OBJECT IDENTIFIER,
//!     signData           BIT STRING
//! }
//! TBS_Sign ::= SEQUENCE {
//!     header              SES_Header,
//!     signatureAlgorithm  OBJECT IDENTIFIER,
//!     seal                SESeal
//! }
//! SES_Signature ::= SEQUENCE {
//!     version     INTEGER,
//!     seal        SESeal,
//!     signInfo    SES_SignInfo
//! }
//! ```

use super::{
    DerResult, TAG_IA5_STRING, TAG_INTEGER, TAG_OBJECT_IDENTIFIER, decode_oid, decode_sequence,
    decode_tlv, decode_uint, encode_bit_string, encode_ia5_string, encode_integer,
    encode_octet_string, encode_oid, encode_printable_string, encode_sequence, encode_utc_time,
    expect_tlv,
};

// ── 名字变体别名（对应 Java 带下划线命名） ─────────────────────────────

#[allow(non_camel_case_types)]
mod java_name_aliases {
    use super::{
        SESHeader, SESPictureInfo, SESPropertyInfo, SESSignature, SealInfo, SignInfo, TBSSign,
    };

    /// 别名：对应 Java `org.ofdrw.gm.ses.v1.TBS_Sign`。
    pub type TBS_Sign = TBSSign;
    /// 别名：对应 Java `org.ofdrw.gm.ses.v1.SES_Signature`。
    pub type SES_Signature = SESSignature;
    /// 别名：对应 Java `org.ofdrw.gm.ses.v1.SES_ESPropertyInfo`。
    pub type SES_ESPropertyInfo = SESPropertyInfo;
    /// 别名：对应 Java `org.ofdrw.gm.ses.v1.SES_SealInfo`。
    pub type SES_SealInfo = SealInfo;
    /// 别名：对应 Java `org.ofdrw.gm.ses.v1.SES_ESPictrueInfo`（Java 保留拼写错误）。
    pub type SES_ESPictrueInfo = SESPictureInfo;
    /// 别名：对应 Java `org.ofdrw.gm.ses.v1.SES_Header`。
    pub type SES_Header = SESHeader;
    /// 别名：对应 Java `org.ofdrw.gm.ses.v1.SES_SignInfo`。
    pub type SES_SignInfo = SignInfo;
}
pub use java_name_aliases::*;

// ── 辅助 ──────────────────────────────────────────────────────────────

/// 将 value 字节重新包装为完整 SEQUENCE DER 编码。
fn repack_sequence(inner: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(inner.len() + 4);
    encode_sequence(inner, &mut out);
    out
}

/// 用闭包构建 SEQUENCE 内部内容后包装为完整 DER。
fn build_sequence<F: FnOnce(&mut Vec<u8>)>(f: F) -> Vec<u8> {
    let mut inner = Vec::new();
    f(&mut inner);
    repack_sequence(&inner)
}

// ── 结构体定义 ────────────────────────────────────────────────────────

/// V1 印章头信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.SES_Header`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESHeader {
    /// 固定值 "ES"。
    pub id: String,
    /// 版本号，V1 固定为 1。
    pub version: u64,
    /// 厂商标识 URI。
    pub vid: String,
}

/// V1 印章属性信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.SES_ESPropertyInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESPropertyInfo {
    /// 印章类型（0=公章, 1=私章, 2=签名章, 3=法人章）。
    pub seal_type: u64,
    /// 印章名称。
    pub name: String,
    /// 证书列表（每个元素为 DER 编码的 X.509 证书）。
    pub cert_list: Vec<Vec<u8>>,
    /// 创建日期，UTCTime 格式 "YYMMDDHHmmSSZ"。
    pub create_date: String,
    /// 有效起始日期，UTCTime 格式。
    pub valid_start: String,
    /// 有效截止日期，UTCTime 格式。
    pub valid_end: String,
}

/// V1 印章图片信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.SES_ESPictureInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESPictureInfo {
    /// 图片类型标识（如 "PNG"）。
    pub pic_type: String,
    /// 图片原始数据。
    pub data: Vec<u8>,
    /// 图片宽度（像素）。
    pub width: u64,
    /// 图片高度（像素）。
    pub height: u64,
}

/// V1 印章信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.SES_SealInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealInfo {
    /// 印章头。
    pub header: SESHeader,
    /// 印章 ID（唯一标识）。
    pub es_id: String,
    /// 印章属性。
    pub property: SESPropertyInfo,
    /// 印章图片。
    pub picture: SESPictureInfo,
}

/// V1 签名信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.SES_SignInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignInfo {
    /// 签名者证书（DER 编码）。
    pub cert: Vec<u8>,
    /// 签名算法 OID 弧段。
    pub signature_algorithm: Vec<u32>,
    /// 签名数据。
    pub sign_data: Vec<u8>,
}

/// V1 电子印章。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.SESeal`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESeal {
    /// 印章信息。
    pub eseal_info: SealInfo,
    /// 签名信息。
    pub sign_info: SignInfo,
}

/// V1 待签名数据。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.TBS_Sign`
///
/// ```asn1
/// TBS_Sign ::= SEQUENCE {
///     version             INTEGER,
///     eseal               SESeal,
///     timeInfo            BIT STRING,
///     dataHash            BIT STRING,
///     propertyInfo        IA5String,
///     cert                OCTET STRING,
///     signatureAlgorithm  OBJECT IDENTIFIER
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TBSSign {
    /// 版本号，V1 固定为 1。
    pub version: u64,
    /// 电子印章。
    pub seal: SESeal,
    /// 签章时间信息。
    pub time_info: Vec<u8>,
    /// 原文杂凑值（SM3 摘要）。
    pub data_hash: Vec<u8>,
    /// 签章属性信息。
    pub property_info: String,
    /// 签章者证书（DER 编码）。
    pub cert: Vec<u8>,
    /// 签名算法 OID 弧段。
    pub signature_algorithm: Vec<u32>,
}

/// V1 印章签名。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.SES_Signature`
///
/// ```asn1
/// SES_Signature ::= SEQUENCE {
///     toSign      TBS_Sign,
///     signature   BIT STRING
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESSignature {
    /// 待签名数据（含 cert 和 signatureAlgorithm）。
    pub to_sign: TBSSign,
    /// 签名数据。
    pub sign_data: Vec<u8>,
}

// ── 编码/解码实现 ─────────────────────────────────────────────────────

impl SESHeader {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            encode_ia5_string(&self.id, inner);
            encode_integer(self.version, inner);
            encode_ia5_string(&self.vid, inner);
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (id_val, next) = expect_tlv(&val, pos, TAG_IA5_STRING)?;
        let id = String::from_utf8_lossy(&id_val).into_owned();
        pos = next;

        let (ver_val, next) = expect_tlv(&val, pos, TAG_INTEGER)?;
        let version = decode_uint(&ver_val);
        pos = next;

        let (vid_val, _) = expect_tlv(&val, pos, TAG_IA5_STRING)?;
        let vid = String::from_utf8_lossy(&vid_val).into_owned();

        Ok(Self { id, version, vid })
    }
}

impl SESPropertyInfo {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            encode_integer(self.seal_type, inner);
            encode_printable_string(&self.name, inner);
            // certList ::= SEQUENCE OF OCTET STRING
            let mut cert_seq = Vec::new();
            for cert in &self.cert_list {
                encode_octet_string(cert, &mut cert_seq);
            }
            encode_sequence(&cert_seq, inner);
            encode_utc_time(&self.create_date, inner);
            encode_utc_time(&self.valid_start, inner);
            encode_utc_time(&self.valid_end, inner);
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (type_val, next) = expect_tlv(&val, pos, TAG_INTEGER)?;
        let seal_type = decode_uint(&type_val);
        pos = next;

        // name: PrintableString (0x13) 或 VisibleString (0x1A)
        let (_tag, name_val, next) = decode_tlv(&val, pos)?;
        let name = String::from_utf8_lossy(&name_val).into_owned();
        pos = next;

        // certList SEQUENCE
        let (cert_seq_val, next) = decode_sequence(&val, pos)?;
        let mut cert_list = Vec::new();
        let mut cpos = 0;
        while cpos < cert_seq_val.len() {
            let (cert_val, cnext) = expect_tlv(&cert_seq_val, cpos, 0x04)?;
            cert_list.push(cert_val);
            cpos = cnext;
        }
        pos = next;

        // UTCTime 字段 (tag 0x17)
        let (create_val, next) = expect_tlv(&val, pos, 0x17)?;
        let create_date = String::from_utf8_lossy(&create_val).into_owned();
        pos = next;

        let (start_val, next) = expect_tlv(&val, pos, 0x17)?;
        let valid_start = String::from_utf8_lossy(&start_val).into_owned();
        pos = next;

        let (end_val, _) = expect_tlv(&val, pos, 0x17)?;
        let valid_end = String::from_utf8_lossy(&end_val).into_owned();

        Ok(Self {
            seal_type,
            name,
            cert_list,
            create_date,
            valid_start,
            valid_end,
        })
    }
}

impl SESPictureInfo {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            encode_printable_string(&self.pic_type, inner);
            encode_octet_string(&self.data, inner);
            encode_integer(self.width, inner);
            encode_integer(self.height, inner);
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (_tag, type_val, next) = decode_tlv(&val, pos)?;
        let pic_type = String::from_utf8_lossy(&type_val).into_owned();
        pos = next;

        let (data_val, next) = expect_tlv(&val, pos, 0x04)?;
        let data = data_val;
        pos = next;

        let (w_val, next) = expect_tlv(&val, pos, TAG_INTEGER)?;
        let width = decode_uint(&w_val);
        pos = next;

        let (h_val, _) = expect_tlv(&val, pos, TAG_INTEGER)?;
        let height = decode_uint(&h_val);

        Ok(Self {
            pic_type,
            data,
            width,
            height,
        })
    }
}

impl SealInfo {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            inner.extend_from_slice(&self.header.encode_der());
            encode_ia5_string(&self.es_id, inner);
            inner.extend_from_slice(&self.property.encode_der());
            inner.extend_from_slice(&self.picture.encode_der());
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (hdr_seq, next) = decode_sequence(&val, pos)?;
        let header = SESHeader::decode_der(&repack_sequence(&hdr_seq))?;
        pos = next;

        let (id_val, next) = expect_tlv(&val, pos, TAG_IA5_STRING)?;
        let es_id = String::from_utf8_lossy(&id_val).into_owned();
        pos = next;

        let (prop_seq, next) = decode_sequence(&val, pos)?;
        let property = SESPropertyInfo::decode_der(&repack_sequence(&prop_seq))?;
        pos = next;

        let (pic_seq, _) = decode_sequence(&val, pos)?;
        let picture = SESPictureInfo::decode_der(&repack_sequence(&pic_seq))?;

        Ok(Self {
            header,
            es_id,
            property,
            picture,
        })
    }
}

impl SignInfo {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            encode_octet_string(&self.cert, inner);
            encode_oid(&self.signature_algorithm, inner);
            encode_bit_string(&self.sign_data, inner);
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (cert_val, next) = expect_tlv(&val, pos, 0x04)?;
        let cert = cert_val;
        pos = next;

        let (oid_val, next) = expect_tlv(&val, pos, TAG_OBJECT_IDENTIFIER)?;
        let signature_algorithm = decode_oid(&oid_val)?;
        pos = next;

        let (sig_val, _) = expect_tlv(&val, pos, 0x03)?;
        // BIT STRING 的第一个字节是 unused-bits 计数
        let sign_data = if sig_val.is_empty() {
            Vec::new()
        } else {
            sig_val[1..].to_vec()
        };

        Ok(Self {
            cert,
            signature_algorithm,
            sign_data,
        })
    }
}

impl SESeal {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            inner.extend_from_slice(&self.eseal_info.encode_der());
            inner.extend_from_slice(&self.sign_info.encode_der());
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (eseal_seq, next) = decode_sequence(&val, pos)?;
        let eseal_info = SealInfo::decode_der(&repack_sequence(&eseal_seq))?;
        pos = next;

        let (sign_seq, _) = decode_sequence(&val, pos)?;
        let sign_info = SignInfo::decode_der(&repack_sequence(&sign_seq))?;

        Ok(Self {
            eseal_info,
            sign_info,
        })
    }
}

impl TBSSign {
    /// DER 编码。
    ///
    /// 对应 Java: `org.ofdrw.gm.ses.v1.TBS_Sign#toASN1Primitive`
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            encode_integer(self.version, inner);
            inner.extend_from_slice(&self.seal.encode_der());
            encode_bit_string(&self.time_info, inner);
            encode_bit_string(&self.data_hash, inner);
            encode_ia5_string(&self.property_info, inner);
            encode_octet_string(&self.cert, inner);
            encode_oid(&self.signature_algorithm, inner);
        })
    }

    /// DER 解码。
    ///
    /// 对应 Java: `org.ofdrw.gm.ses.v1.TBS_Sign(ASN1Sequence)`
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (ver_val, next) = expect_tlv(&val, pos, TAG_INTEGER)?;
        let version = decode_uint(&ver_val);
        pos = next;

        let (seal_seq, next) = decode_sequence(&val, pos)?;
        let seal = SESeal::decode_der(&repack_sequence(&seal_seq))?;
        pos = next;

        let (time_val, next) = expect_tlv(&val, pos, 0x03)?;
        let time_info = if time_val.is_empty() {
            Vec::new()
        } else {
            time_val[1..].to_vec()
        };
        pos = next;

        let (hash_val, next) = expect_tlv(&val, pos, 0x03)?;
        let data_hash = if hash_val.is_empty() {
            Vec::new()
        } else {
            hash_val[1..].to_vec()
        };
        pos = next;

        // propertyInfo: IA5String (0x16) 或兼容 DERIA5String
        let (prop_val, next) = expect_tlv(&val, pos, TAG_IA5_STRING)?;
        let property_info = String::from_utf8_lossy(&prop_val).into_owned();
        pos = next;

        let (cert_val, next) = expect_tlv(&val, pos, 0x04)?;
        let cert = cert_val;
        pos = next;

        let (oid_val, _next) = expect_tlv(&val, pos, TAG_OBJECT_IDENTIFIER)?;
        let signature_algorithm = decode_oid(&oid_val)?;

        Ok(Self {
            version,
            seal,
            time_info,
            data_hash,
            property_info,
            cert,
            signature_algorithm,
        })
    }
}

impl SESSignature {
    /// DER 编码。
    ///
    /// 对应 Java: `org.ofdrw.gm.ses.v1.SES_Signature#toASN1Primitive`
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            inner.extend_from_slice(&self.to_sign.encode_der());
            encode_bit_string(&self.sign_data, inner);
        })
    }

    /// DER 解码。
    ///
    /// 对应 Java: `org.ofdrw.gm.ses.v1.SES_Signature(ASN1Sequence)`
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        // TBS_Sign（SEQUENCE）
        let (tbs_seq, next) = decode_sequence(&val, pos)?;
        let to_sign = TBSSign::decode_der(&repack_sequence(&tbs_seq))?;
        pos = next;

        // signature（BIT STRING）
        let (sig_val, _next) = expect_tlv(&val, pos, 0x03)?;
        let sign_data = if sig_val.is_empty() {
            Vec::new()
        } else {
            sig_val[1..].to_vec()
        };

        Ok(Self { to_sign, sign_data })
    }
}

// ── 测试 ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::TAG_SEQUENCE;
    use super::*;

    /// SM2-with-SM3 签名算法 OID: 1.2.156.10197.1.501
    const SM2_SM3_OID: &[u32] = &[1, 2, 156, 10_197, 1, 501];

    fn sample_header() -> SESHeader {
        SESHeader {
            id: "ES".into(),
            version: 1,
            vid: "http://www.ofdrw.org".into(),
        }
    }

    fn sample_property() -> SESPropertyInfo {
        SESPropertyInfo {
            seal_type: 0,
            name: "TestSeal".into(),
            cert_list: vec![vec![0x30, 0x03, 0x02, 0x01, 0x01]],
            create_date: "250101000000Z".into(),
            valid_start: "250101000000Z".into(),
            valid_end: "300101000000Z".into(),
        }
    }

    fn sample_picture() -> SESPictureInfo {
        SESPictureInfo {
            pic_type: "PNG".into(),
            data: vec![0x89, 0x50, 0x4E, 0x47],
            width: 200,
            height: 200,
        }
    }

    fn sample_seal_info() -> SealInfo {
        SealInfo {
            header: sample_header(),
            es_id: "ES0001".into(),
            property: sample_property(),
            picture: sample_picture(),
        }
    }

    fn sample_sign_info() -> SignInfo {
        SignInfo {
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xAA; 64],
        }
    }

    fn sample_seseal() -> SESeal {
        SESeal {
            eseal_info: sample_seal_info(),
            sign_info: sample_sign_info(),
        }
    }

    #[test]
    fn seseal_encode_decode_roundtrip() {
        let seal = sample_seseal();
        let der = seal.encode_der();
        let decoded = SESeal::decode_der(&der).unwrap();
        assert_eq!(decoded, seal);
    }

    #[test]
    fn seseal_field_values_preserved() {
        let seal = sample_seseal();
        let der = seal.encode_der();
        let decoded = SESeal::decode_der(&der).unwrap();
        assert_eq!(decoded.eseal_info.header.id, "ES");
        assert_eq!(decoded.eseal_info.header.version, 1);
        assert_eq!(decoded.eseal_info.header.vid, "http://www.ofdrw.org");
        assert_eq!(decoded.eseal_info.es_id, "ES0001");
        assert_eq!(decoded.eseal_info.property.seal_type, 0);
        assert_eq!(decoded.eseal_info.property.name, "TestSeal");
        assert_eq!(decoded.eseal_info.picture.width, 200);
        assert_eq!(decoded.eseal_info.picture.height, 200);
        assert_eq!(decoded.sign_info.signature_algorithm, SM2_SM3_OID);
        assert_eq!(decoded.sign_info.sign_data, vec![0xAA; 64]);
    }

    #[test]
    fn seseal_starts_with_sequence_tag() {
        let seal = sample_seseal();
        let der = seal.encode_der();
        assert_eq!(der[0], TAG_SEQUENCE);
    }

    #[test]
    fn ses_signature_roundtrip() {
        let sig = SESSignature {
            to_sign: TBSSign {
                version: 1,
                seal: sample_seseal(),
                time_info: b"2025-01-01 00:00:00".to_vec(),
                data_hash: vec![0xAA; 32],
                property_info: "test".into(),
                cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
                signature_algorithm: SM2_SM3_OID.to_vec(),
            },
            sign_data: vec![0xBB; 64],
        };
        let der = sig.encode_der();
        let decoded = SESSignature::decode_der(&der).unwrap();
        assert_eq!(decoded, sig);
    }

    #[test]
    fn tbs_sign_roundtrip() {
        let tbs = TBSSign {
            version: 1,
            seal: sample_seseal(),
            time_info: b"2025-01-01 00:00:00".to_vec(),
            data_hash: vec![0xAA; 32],
            property_info: "test-property".into(),
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
        };
        let der = tbs.encode_der();
        let decoded = TBSSign::decode_der(&der).unwrap();
        assert_eq!(decoded, tbs);
    }

    #[test]
    fn property_multiple_certs_roundtrip() {
        let mut prop = sample_property();
        prop.cert_list = vec![
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
            vec![0x30, 0x05, 0x02, 0x01, 0x02, 0x02, 0x01, 0x03],
        ];
        let der = prop.encode_der();
        let decoded = SESPropertyInfo::decode_der(&der).unwrap();
        assert_eq!(decoded.cert_list.len(), 2);
        assert_eq!(
            decoded.cert_list[1],
            vec![0x30, 0x05, 0x02, 0x01, 0x02, 0x02, 0x01, 0x03]
        );
    }

    #[test]
    fn sign_info_empty_sign_data_roundtrip() {
        let mut si = sample_sign_info();
        si.sign_data = Vec::new();
        let der = si.encode_der();
        let decoded = SignInfo::decode_der(&der).unwrap();
        assert!(decoded.sign_data.is_empty());
    }
}
