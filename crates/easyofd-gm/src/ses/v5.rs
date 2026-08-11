//! SES V5 电子印章 ASN.1 结构定义。
//!
//! 对应 Java 版 [`org.ofdrw.gm.ses.v5`](https://github.com/ofdrw/ofdrw) 包。
//! V5 等同 V4 + 可选 `timeStamp` 字段，用于可信时间戳。
//!
//! # 结构概览
//!
//! ```asn1
//! SESeal ::= SEQUENCE {
//!     esealInfo           SES_SealInfo,
//!     cert                OCTET STRING,
//!     signatureAlgorithm  OBJECT IDENTIFIER,
//!     signData            BIT STRING,
//!     timeStamp           [1] EXPLICIT OCTET STRING OPTIONAL
//! }
//! SES_SealInfo ::= SEQUENCE {
//!     header      SES_Header,
//!     esID        IA5String,
//!     property    SES_ESPropertyInfo,
//!     picture     SES_ESPictureInfo,
//!     extDatas    [0] EXPLICIT SEQUENCE OF ExtData OPTIONAL
//! }
//! SES_Header ::= SEQUENCE {
//!     id      IA5String,
//!     version INTEGER,
//!     vid     IA5String
//! }
//! SES_ESPropertyInfo ::= SEQUENCE {
//!     type        INTEGER,
//!     name        PrintableString,
//!     certList    SEQUENCE OF OCTET STRING,
//!     createDate  GeneralizedTime,
//!     validStart  GeneralizedTime,
//!     validEnd    GeneralizedTime
//! }
//! SES_ESPictureInfo ::= SEQUENCE {
//!     type     PrintableString,
//!     data     OCTET STRING,
//!     width    INTEGER,
//!     height   INTEGER
//! }
//! CertDigest ::= SEQUENCE {
//!     cert        OCTET STRING,
//!     digestAlg   OBJECT IDENTIFIER,
//!     digestValue OCTET STRING
//! }
//! TBS_Sign ::= SEQUENCE {
//!     header              SES_Header,
//!     signatureAlgorithm  OBJECT IDENTIFIER,
//!     seal                SESeal,
//!     extDatas            [0] EXPLICIT SEQUENCE OF ExtData OPTIONAL
//! }
//! SES_Signature ::= SEQUENCE {
//!     version     INTEGER,
//!     seal        SESeal,
//!     cert        OCTET STRING,
//!     signatureAlgorithm OBJECT IDENTIFIER,
//!     signData    BIT STRING
//! }
//! ```

use super::{
    DerError, DerResult, TAG_IA5_STRING, TAG_INTEGER, TAG_OBJECT_IDENTIFIER, TAG_SEQUENCE,
    decode_context_explicit_optional, decode_oid, decode_sequence, decode_tlv, decode_uint,
    encode_bit_string, encode_context_explicit, encode_generalized_time, encode_ia5_string,
    encode_integer, encode_octet_string, encode_oid, encode_printable_string, encode_sequence,
    expect_tlv,
};

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

/// V5 印章头信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v5.SES_Header`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESHeader {
    /// 固定值 "ES"。
    pub id: String,
    /// 版本号，V5 固定为 5。
    pub version: u64,
    /// 厂商标识 URI。
    pub vid: String,
}

/// V5 证书摘要信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v5.CertDigest`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertDigest {
    /// 证书（DER 编码）。
    pub cert: Vec<u8>,
    /// 摘要算法 OID 弧段。
    pub digest_alg: Vec<u32>,
    /// 摘要值。
    pub digest_value: Vec<u8>,
}

/// V5 证书列表项（CHOICE）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertChoice {
    /// 完整证书（DER 编码）。
    FullCert(Vec<u8>),
    /// 证书摘要。
    Digest(CertDigest),
}

/// V5 印章属性信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v5.SES_ESPropertyInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESPropertyInfo {
    /// 印章类型。
    pub seal_type: u64,
    /// 印章名称。
    pub name: String,
    /// 证书列表。
    pub cert_list: Vec<CertChoice>,
    /// 创建日期，GeneralizedTime 格式。
    pub create_date: String,
    /// 有效起始日期，GeneralizedTime 格式。
    pub valid_start: String,
    /// 有效截止日期，GeneralizedTime 格式。
    pub valid_end: String,
}

/// V5 印章图片信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v5.SES_ESPictureInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESPictureInfo {
    /// 图片类型标识。
    pub pic_type: String,
    /// 图片原始数据。
    pub data: Vec<u8>,
    /// 图片宽度（像素）。
    pub width: u64,
    /// 图片高度（像素）。
    pub height: u64,
}

/// V5 印章信息。
///
/// 对应 Java: `org.ofdrw.gm.ses.v5.SES_SealInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealInfo {
    /// 印章头。
    pub header: SESHeader,
    /// 印章 ID。
    pub es_id: String,
    /// 印章属性。
    pub property: SESPropertyInfo,
    /// 印章图片。
    pub picture: SESPictureInfo,
}

/// V5 电子印章。
///
/// V5 等同 V4 展平结构 + 可选 `timeStamp`。
///
/// 对应 Java: `org.ofdrw.gm.ses.v5.SESeal`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESeal {
    /// 印章信息。
    pub eseal_info: SealInfo,
    /// 签名者证书（DER 编码）。
    pub cert: Vec<u8>,
    /// 签名算法 OID 弧段。
    pub signature_algorithm: Vec<u32>,
    /// 签名数据。
    pub sign_data: Vec<u8>,
    /// 可选可信时间戳。
    pub time_stamp: Option<Vec<u8>>,
}

/// V5 待签名数据。
///
/// 对应 Java: `org.ofdrw.gm.ses.v5.TBS_Sign`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TBSSign {
    /// 印章头。
    pub header: SESHeader,
    /// 签名算法 OID 弧段。
    pub signature_algorithm: Vec<u32>,
    /// 电子印章。
    pub seal: SESeal,
}

/// V5 印章签名。
///
/// 对应 Java: `org.ofdrw.gm.ses.v5.SES_Signature`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESSignature {
    /// 版本号，V5 固定为 5。
    pub version: u64,
    /// 电子印章。
    pub seal: SESeal,
    /// 签名者证书（DER 编码）。
    pub cert: Vec<u8>,
    /// 签名算法 OID 弧段。
    pub signature_algorithm: Vec<u32>,
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

impl CertDigest {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            encode_octet_string(&self.cert, inner);
            encode_oid(&self.digest_alg, inner);
            encode_octet_string(&self.digest_value, inner);
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
        let digest_alg = decode_oid(&oid_val)?;
        pos = next;

        let (digest_val, _) = expect_tlv(&val, pos, 0x04)?;
        let digest_value = digest_val;

        Ok(Self {
            cert,
            digest_alg,
            digest_value,
        })
    }
}

impl SESPropertyInfo {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            encode_integer(self.seal_type, inner);
            encode_printable_string(&self.name, inner);
            let mut cert_seq = Vec::new();
            for cert in &self.cert_list {
                match cert {
                    CertChoice::FullCert(data) => {
                        encode_octet_string(data, &mut cert_seq);
                    }
                    CertChoice::Digest(digest) => {
                        cert_seq.extend_from_slice(&digest.encode_der());
                    }
                }
            }
            encode_sequence(&cert_seq, inner);
            encode_generalized_time(&self.create_date, inner);
            encode_generalized_time(&self.valid_start, inner);
            encode_generalized_time(&self.valid_end, inner);
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (type_val, next) = expect_tlv(&val, pos, TAG_INTEGER)?;
        let seal_type = decode_uint(&type_val);
        pos = next;

        let (_tag, name_val, next) = decode_tlv(&val, pos)?;
        let name = String::from_utf8_lossy(&name_val).into_owned();
        pos = next;

        let (cert_seq_val, next) = decode_sequence(&val, pos)?;
        let mut cert_list = Vec::new();
        let mut cpos = 0;
        while cpos < cert_seq_val.len() {
            let tag = cert_seq_val[cpos];
            if tag == 0x04 {
                let (cert_val, cnext) = expect_tlv(&cert_seq_val, cpos, 0x04)?;
                cert_list.push(CertChoice::FullCert(cert_val));
                cpos = cnext;
            } else if tag == TAG_SEQUENCE {
                let (digest_seq, cnext) = decode_sequence(&cert_seq_val, cpos)?;
                let digest = CertDigest::decode_der(&repack_sequence(&digest_seq))?;
                cert_list.push(CertChoice::Digest(digest));
                cpos = cnext;
            } else {
                return Err(DerError("unexpected tag in certList"));
            }
        }
        pos = next;

        let (create_val, next) = expect_tlv(&val, pos, 0x18)?;
        let create_date = String::from_utf8_lossy(&create_val).into_owned();
        pos = next;

        let (start_val, next) = expect_tlv(&val, pos, 0x18)?;
        let valid_start = String::from_utf8_lossy(&start_val).into_owned();
        pos = next;

        let (end_val, _) = expect_tlv(&val, pos, 0x18)?;
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

impl SESeal {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            inner.extend_from_slice(&self.eseal_info.encode_der());
            encode_octet_string(&self.cert, inner);
            encode_oid(&self.signature_algorithm, inner);
            encode_bit_string(&self.sign_data, inner);
            // 可选 timeStamp [1] EXPLICIT OCTET STRING
            if let Some(ref ts) = self.time_stamp {
                let mut ts_inner = Vec::new();
                encode_octet_string(ts, &mut ts_inner);
                encode_context_explicit(1, &ts_inner, inner);
            }
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (eseal_seq, next) = decode_sequence(&val, pos)?;
        let eseal_info = SealInfo::decode_der(&repack_sequence(&eseal_seq))?;
        pos = next;

        let (cert_val, next) = expect_tlv(&val, pos, 0x04)?;
        let cert = cert_val;
        pos = next;

        let (oid_val, next) = expect_tlv(&val, pos, TAG_OBJECT_IDENTIFIER)?;
        let signature_algorithm = decode_oid(&oid_val)?;
        pos = next;

        let (sig_val, next) = expect_tlv(&val, pos, 0x03)?;
        let sign_data = if sig_val.is_empty() {
            Vec::new()
        } else {
            sig_val[1..].to_vec()
        };
        pos = next;

        // 可选 timeStamp [1] EXPLICIT OCTET STRING
        let (ts_data, _) = decode_context_explicit_optional(&val, pos, 1)?;
        let time_stamp = ts_data.and_then(|inner| {
            // inner 是 [1] 的 value 部分，其中包含一个 OCTET STRING
            if inner.is_empty() {
                return None;
            }
            // 解码内部的 OCTET STRING
            if inner[0] == 0x04 {
                expect_tlv(&inner, 0, 0x04).ok().map(|(val, _)| val)
            } else {
                Some(inner)
            }
        });

        Ok(Self {
            eseal_info,
            cert,
            signature_algorithm,
            sign_data,
            time_stamp,
        })
    }
}

impl TBSSign {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            inner.extend_from_slice(&self.header.encode_der());
            encode_oid(&self.signature_algorithm, inner);
            inner.extend_from_slice(&self.seal.encode_der());
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (hdr_seq, next) = decode_sequence(&val, pos)?;
        let header = SESHeader::decode_der(&repack_sequence(&hdr_seq))?;
        pos = next;

        let (oid_val, next) = expect_tlv(&val, pos, TAG_OBJECT_IDENTIFIER)?;
        let signature_algorithm = decode_oid(&oid_val)?;
        pos = next;

        let (seal_seq, _) = decode_sequence(&val, pos)?;
        let seal = SESeal::decode_der(&repack_sequence(&seal_seq))?;

        Ok(Self {
            header,
            signature_algorithm,
            seal,
        })
    }
}

impl SESSignature {
    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        build_sequence(|inner| {
            encode_integer(self.version, inner);
            inner.extend_from_slice(&self.seal.encode_der());
            encode_octet_string(&self.cert, inner);
            encode_oid(&self.signature_algorithm, inner);
            encode_bit_string(&self.sign_data, inner);
        })
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (ver_val, next) = expect_tlv(&val, pos, TAG_INTEGER)?;
        let version = decode_uint(&ver_val);
        pos = next;

        let (seal_seq, next) = decode_sequence(&val, pos)?;
        let seal = SESeal::decode_der(&repack_sequence(&seal_seq))?;
        pos = next;

        let (cert_val, next) = expect_tlv(&val, pos, 0x04)?;
        let cert = cert_val;
        pos = next;

        let (oid_val, next) = expect_tlv(&val, pos, TAG_OBJECT_IDENTIFIER)?;
        let signature_algorithm = decode_oid(&oid_val)?;
        pos = next;

        let (sig_val, _) = expect_tlv(&val, pos, 0x03)?;
        let sign_data = if sig_val.is_empty() {
            Vec::new()
        } else {
            sig_val[1..].to_vec()
        };

        Ok(Self {
            version,
            seal,
            cert,
            signature_algorithm,
            sign_data,
        })
    }
}

// ── 测试 ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SM2_SM3_OID: &[u32] = &[1, 2, 156, 10_197, 1, 501];

    fn sample_header() -> SESHeader {
        SESHeader {
            id: "ES".into(),
            version: 5,
            vid: "http://www.ofdrw.org".into(),
        }
    }

    fn sample_property() -> SESPropertyInfo {
        SESPropertyInfo {
            seal_type: 0,
            name: "V5TestSeal".into(),
            cert_list: vec![CertChoice::FullCert(vec![0x30, 0x03, 0x02, 0x01, 0x01])],
            create_date: "20250101000000Z".into(),
            valid_start: "20250101000000Z".into(),
            valid_end: "20300101000000Z".into(),
        }
    }

    fn sample_picture() -> SESPictureInfo {
        SESPictureInfo {
            pic_type: "PNG".into(),
            data: vec![0x89, 0x50, 0x4E, 0x47],
            width: 400,
            height: 400,
        }
    }

    fn sample_seal_info() -> SealInfo {
        SealInfo {
            header: sample_header(),
            es_id: "ES_V5_001".into(),
            property: sample_property(),
            picture: sample_picture(),
        }
    }

    fn sample_seseal() -> SESeal {
        SESeal {
            eseal_info: sample_seal_info(),
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xDD; 64],
            time_stamp: Some(vec![0x01, 0x02, 0x03, 0x04]),
        }
    }

    fn sample_seseal_no_timestamp() -> SESeal {
        SESeal {
            time_stamp: None,
            ..sample_seseal()
        }
    }

    #[test]
    fn seseal_with_timestamp_roundtrip() {
        let seal = sample_seseal();
        let der = seal.encode_der();
        let decoded = SESeal::decode_der(&der).unwrap();
        assert_eq!(decoded, seal);
    }

    #[test]
    fn seseal_without_timestamp_roundtrip() {
        let seal = sample_seseal_no_timestamp();
        let der = seal.encode_der();
        let decoded = SESeal::decode_der(&der).unwrap();
        assert_eq!(decoded, seal);
        assert!(decoded.time_stamp.is_none());
    }

    #[test]
    fn seseal_flat_fields_preserved() {
        let seal = sample_seseal();
        let der = seal.encode_der();
        let decoded = SESeal::decode_der(&der).unwrap();
        assert_eq!(decoded.cert, seal.cert);
        assert_eq!(decoded.signature_algorithm, SM2_SM3_OID);
        assert_eq!(decoded.sign_data, vec![0xDD; 64]);
        assert_eq!(decoded.eseal_info.header.id, "ES");
        assert_eq!(decoded.eseal_info.header.version, 5);
        assert_eq!(decoded.eseal_info.es_id, "ES_V5_001");
        assert_eq!(decoded.eseal_info.picture.width, 400);
        assert_eq!(decoded.eseal_info.picture.height, 400);
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
            version: 5,
            seal: sample_seseal(),
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xEE; 32],
        };
        let der = sig.encode_der();
        let decoded = SESSignature::decode_der(&der).unwrap();
        assert_eq!(decoded, sig);
    }

    #[test]
    fn tbs_sign_roundtrip() {
        let tbs = TBSSign {
            header: sample_header(),
            signature_algorithm: SM2_SM3_OID.to_vec(),
            seal: sample_seseal(),
        };
        let der = tbs.encode_der();
        let decoded = TBSSign::decode_der(&der).unwrap();
        assert_eq!(decoded, tbs);
    }

    #[test]
    fn empty_sign_data_roundtrip() {
        let mut seal = sample_seseal_no_timestamp();
        seal.sign_data = Vec::new();
        let der = seal.encode_der();
        let decoded = SESeal::decode_der(&der).unwrap();
        assert!(decoded.sign_data.is_empty());
    }
}
