//! SES 版本解析器。
//!
//! 对应 Java: `org.ofdrw.gm.ses.parse.VersionParser`

use super::{SESVersion, SESVersionHolder};
use crate::ses::{DerError, DerResult, decode_sequence, decode_tlv, decode_uint};

/// 版本解析器，从原始 DER 字节自动探测 SES 版本。
///
/// 对应 Java: `org.ofdrw.gm.ses.parse.VersionParser`
pub struct VersionParser;

impl VersionParser {
    /// 从印章 DER 字节解析版本（SESeal 结构）。
    ///
    /// 对应 Java: `parseSES_SealVersion`
    ///
    /// 根据顶层 SEQUENCE 的子元素数量判断版本：
    /// - 2 个子元素 → V1（`SESeal { esealInfo, signInfo }`）
    /// - 4 个子元素 → V4 或 V5（通过 header version 区分）
    /// - 5 个子元素 → V5（含 timeStamp）
    ///
    /// # 错误
    ///
    /// 结构不合法或版本无法识别时返回错误。
    pub fn parse_seal_version(der: &[u8]) -> DerResult<SESVersionHolder> {
        let (seq_val, _) = decode_sequence(der, 0)?;
        let child_count = count_sequence_children(&seq_val)?;
        let version = match child_count {
            2 => SESVersion::V1,
            5 => SESVersion::V5,
            4 => {
                // 通过 header version 区分 V4 / V5
                let header_ver = Self::extract_header_version(&seq_val)?;
                if header_ver == 5 {
                    SESVersion::V5
                } else {
                    SESVersion::V4
                }
            }
            _ => return Err(DerError("未知的印章结构，无法匹配任何已知版本")),
        };
        Ok(SESVersionHolder::new(version, seq_val))
    }

    /// 从签章 DER 字节解析版本（SES_Signature 结构）。
    ///
    /// 对应 Java: `parseSES_SignatureVersion`
    ///
    /// SES_Signature 的顶层 SEQUENCE 包含：
    /// - V1: TBS_Sign, signature（2 个子元素）
    /// - V4/V5: TBS_Sign, cert, signatureAlgorithm, signData（4 个子元素）
    ///
    /// 版本号从 TBS_Sign 内部的 version INTEGER 读取。
    ///
    /// # 错误
    ///
    /// 结构不合法或版本无法识别时返回错误。
    pub fn parse_signature_version(der: &[u8]) -> DerResult<SESVersionHolder> {
        let (seq_val, _) = decode_sequence(der, 0)?;
        let child_count = count_sequence_children(&seq_val)?;
        let version = match child_count {
            2 => SESVersion::V1,
            n if n >= 4 => {
                // V4/V5: 从 TBS_Sign 的 version 字段读取
                let tbs_ver = Self::extract_tbs_sign_version(&seq_val)?;
                if tbs_ver == 5 {
                    SESVersion::V5
                } else {
                    SESVersion::V4
                }
            }
            _ => return Err(DerError("未知的签章结构，无法匹配任何已知版本")),
        };
        Ok(SESVersionHolder::new(version, seq_val))
    }

    /// 从印章结构中提取 header 的 version 字段。
    ///
    /// 结构: `SESeal → esealInfo(SEQUENCE) → header(SEQUENCE) → version(INTEGER)`
    fn extract_header_version(seal_seq: &[u8]) -> DerResult<u32> {
        // seal_seq 是 SESeal 内部的字节
        // 第一个子元素是 esealInfo (SEQUENCE)
        let (eseal_info_val, _) = {
            let (tag, val, _) = decode_tlv(seal_seq, 0)?;
            if tag != 0x30 {
                return Err(DerError("期望 esealInfo 为 SEQUENCE"));
            }
            (val, 0)
        };
        // esealInfo 内部第一个子元素是 header (SEQUENCE)
        let (header_val, _) = {
            let (tag, val, _) = decode_tlv(&eseal_info_val, 0)?;
            if tag != 0x30 {
                return Err(DerError("期望 header 为 SEQUENCE"));
            }
            (val, 0)
        };
        // header 内部第二个子元素是 version (INTEGER)
        // 跳过第一个子元素 (id: IA5String)
        let (_, _, hpos) = decode_tlv(&header_val, 0)?;
        // 读取 version (INTEGER)
        let (tag, ver_val, _) = decode_tlv(&header_val, hpos)?;
        if tag != 0x02 {
            return Err(DerError("期望 version 为 INTEGER"));
        }
        #[allow(clippy::cast_possible_truncation)]
        let ver = decode_uint(&ver_val) as u32;
        Ok(ver)
    }

    /// 从 SES_Signature 结构中提取 TBS_Sign 的 version 字段。
    ///
    /// SES_Signature 首元素是 TBS_Sign SEQUENCE，
    /// TBS_Sign 首元素是 version INTEGER。
    fn extract_tbs_sign_version(sig_seq: &[u8]) -> DerResult<u32> {
        // 第一个子元素是 TBS_Sign (SEQUENCE)
        let (tbs_val, _) = {
            let (tag, val, _) = decode_tlv(sig_seq, 0)?;
            if tag != 0x30 {
                return Err(DerError("期望 TBS_Sign 为 SEQUENCE"));
            }
            (val, 0)
        };
        // TBS_Sign 第一个子元素是 version (INTEGER)
        let (tag, ver_val, _) = decode_tlv(&tbs_val, 0)?;
        if tag != 0x02 {
            return Err(DerError("期望 TBS_Sign.version 为 INTEGER"));
        }
        #[allow(clippy::cast_possible_truncation)]
        let ver = decode_uint(&ver_val) as u32;
        Ok(ver)
    }
}

/// 计算 SEQUENCE 内部的顶层子元素数量。
fn count_sequence_children(bytes: &[u8]) -> DerResult<usize> {
    let mut count = 0;
    let mut pos = 0;
    while pos < bytes.len() {
        let (_, _, next) = decode_tlv(bytes, pos)?;
        count += 1;
        pos = next;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ses::v1::{self, SESSignature as V1Sig};
    use crate::ses::v4::{self, SESSignature as V4Sig};
    use crate::ses::v5::{self, SESSignature as V5Sig};

    const SM2_SM3_OID: &[u32] = &[1, 2, 156, 10_197, 1, 501];

    fn make_v1_signature_der() -> Vec<u8> {
        let sig = V1Sig {
            to_sign: v1::TBSSign {
                version: 1,
                seal: v1::SESeal {
                    eseal_info: v1::SealInfo {
                        header: v1::SESHeader {
                            id: "ES".into(),
                            version: 1,
                            vid: "test".into(),
                        },
                        es_id: "ES001".into(),
                        property: v1::SESPropertyInfo {
                            seal_type: 0,
                            name: "Test".into(),
                            cert_list: vec![vec![0x01]],
                            create_date: "250101000000Z".into(),
                            valid_start: "250101000000Z".into(),
                            valid_end: "300101000000Z".into(),
                        },
                        picture: v1::SESPictureInfo {
                            pic_type: "PNG".into(),
                            data: vec![0x01],
                            width: 100,
                            height: 100,
                        },
                    },
                    sign_info: v1::SignInfo {
                        cert: vec![0x01],
                        signature_algorithm: SM2_SM3_OID.to_vec(),
                        sign_data: vec![0xAA; 32],
                    },
                },
                time_info: b"2025-01-01 00:00:00".to_vec(),
                data_hash: vec![0xBB; 32],
                property_info: "test".into(),
                cert: vec![0x01],
                signature_algorithm: SM2_SM3_OID.to_vec(),
            },
            sign_data: vec![0xBB; 32],
        };
        sig.encode_der()
    }

    fn make_v4_signature_der() -> Vec<u8> {
        let sig = V4Sig {
            to_sign: v4::TBSSign {
                version: 4,
                seal: v4::SESeal {
                    eseal_info: v4::SealInfo {
                        header: v4::SESHeader {
                            id: "ES".into(),
                            version: 4,
                            vid: "test".into(),
                        },
                        es_id: "ES004".into(),
                        property: v4::SESPropertyInfo {
                            seal_type: 0,
                            name: "Test".into(),
                            cert_list: vec![v4::CertChoice::FullCert(vec![0x01])],
                            create_date: "20250101000000Z".into(),
                            valid_start: "20250101000000Z".into(),
                            valid_end: "20300101000000Z".into(),
                        },
                        picture: v4::SESPictureInfo {
                            pic_type: "PNG".into(),
                            data: vec![0x01],
                            width: 100,
                            height: 100,
                        },
                    },
                    cert: vec![0x01],
                    signature_algorithm: SM2_SM3_OID.to_vec(),
                    sign_data: vec![0xCC; 32],
                },
                time_info: "20250101000000Z".into(),
                data_hash: vec![0xDD; 32],
                property_info: "test".into(),
            },
            cert: vec![0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xDD; 32],
        };
        sig.encode_der()
    }

    fn make_v5_signature_der() -> Vec<u8> {
        let sig = V5Sig {
            to_sign: v5::TBSSign {
                version: 5,
                seal: v5::SESeal {
                    eseal_info: v5::SealInfo {
                        header: v5::SESHeader {
                            id: "ES".into(),
                            version: 5,
                            vid: "test".into(),
                        },
                        es_id: "ES005".into(),
                        property: v5::SESPropertyInfo {
                            seal_type: 0,
                            name: "Test".into(),
                            cert_list: vec![v5::CertChoice::FullCert(vec![0x01])],
                            create_date: "20250101000000Z".into(),
                            valid_start: "20250101000000Z".into(),
                            valid_end: "20300101000000Z".into(),
                        },
                        picture: v5::SESPictureInfo {
                            pic_type: "PNG".into(),
                            data: vec![0x01],
                            width: 100,
                            height: 100,
                        },
                    },
                    cert: vec![0x01],
                    signature_algorithm: SM2_SM3_OID.to_vec(),
                    sign_data: vec![0xEE; 32],
                    time_stamp: None,
                },
                time_info: "20250101000000Z".into(),
                data_hash: vec![0xFF; 32],
                property_info: "test".into(),
            },
            cert: vec![0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xFF; 32],
            time_stamp: None,
        };
        sig.encode_der()
    }

    #[test]
    fn parse_v1_signature_version() {
        let der = make_v1_signature_der();
        let holder = VersionParser::parse_signature_version(&der).unwrap();
        assert_eq!(holder.version(), SESVersion::V1);
    }

    #[test]
    fn parse_v4_signature_version() {
        let der = make_v4_signature_der();
        let holder = VersionParser::parse_signature_version(&der).unwrap();
        assert_eq!(holder.version(), SESVersion::V4);
    }

    #[test]
    fn parse_v5_signature_version() {
        let der = make_v5_signature_der();
        let holder = VersionParser::parse_signature_version(&der).unwrap();
        assert_eq!(holder.version(), SESVersion::V5);
    }

    #[test]
    fn parse_seal_version_v1() {
        // 构造 V1 SESeal DER
        let seal = v1::SESeal {
            eseal_info: v1::SealInfo {
                header: v1::SESHeader {
                    id: "ES".into(),
                    version: 1,
                    vid: "test".into(),
                },
                es_id: "ES001".into(),
                property: v1::SESPropertyInfo {
                    seal_type: 0,
                    name: "Test".into(),
                    cert_list: vec![vec![0x01]],
                    create_date: "250101000000Z".into(),
                    valid_start: "250101000000Z".into(),
                    valid_end: "300101000000Z".into(),
                },
                picture: v1::SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x01],
                    width: 100,
                    height: 100,
                },
            },
            sign_info: v1::SignInfo {
                cert: vec![0x01],
                signature_algorithm: SM2_SM3_OID.to_vec(),
                sign_data: vec![0xAA; 32],
            },
        };
        let der = seal.encode_der();
        let holder = VersionParser::parse_seal_version(&der).unwrap();
        assert_eq!(holder.version(), SESVersion::V1);
    }

    #[test]
    fn parse_seal_version_v4() {
        let seal = v4::SESeal {
            eseal_info: v4::SealInfo {
                header: v4::SESHeader {
                    id: "ES".into(),
                    version: 4,
                    vid: "test".into(),
                },
                es_id: "ES004".into(),
                property: v4::SESPropertyInfo {
                    seal_type: 0,
                    name: "Test".into(),
                    cert_list: vec![v4::CertChoice::FullCert(vec![0x01])],
                    create_date: "20250101000000Z".into(),
                    valid_start: "20250101000000Z".into(),
                    valid_end: "20300101000000Z".into(),
                },
                picture: v4::SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x01],
                    width: 100,
                    height: 100,
                },
            },
            cert: vec![0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xCC; 32],
        };
        let der = seal.encode_der();
        let holder = VersionParser::parse_seal_version(&der).unwrap();
        assert_eq!(holder.version(), SESVersion::V4);
    }

    #[test]
    fn parse_seal_version_v5_no_timestamp() {
        let seal = v5::SESeal {
            eseal_info: v5::SealInfo {
                header: v5::SESHeader {
                    id: "ES".into(),
                    version: 5,
                    vid: "test".into(),
                },
                es_id: "ES005".into(),
                property: v5::SESPropertyInfo {
                    seal_type: 0,
                    name: "Test".into(),
                    cert_list: vec![v5::CertChoice::FullCert(vec![0x01])],
                    create_date: "20250101000000Z".into(),
                    valid_start: "20250101000000Z".into(),
                    valid_end: "20300101000000Z".into(),
                },
                picture: v5::SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x01],
                    width: 100,
                    height: 100,
                },
            },
            cert: vec![0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xEE; 32],
            time_stamp: None,
        };
        let der = seal.encode_der();
        let holder = VersionParser::parse_seal_version(&der).unwrap();
        // V5 无 timeStamp 时顶层 4 个子元素，通过 header version=5 区分
        assert_eq!(holder.version(), SESVersion::V5);
    }

    #[test]
    fn parse_seal_version_v5_with_timestamp() {
        let seal = v5::SESeal {
            eseal_info: v5::SealInfo {
                header: v5::SESHeader {
                    id: "ES".into(),
                    version: 5,
                    vid: "test".into(),
                },
                es_id: "ES005".into(),
                property: v5::SESPropertyInfo {
                    seal_type: 0,
                    name: "Test".into(),
                    cert_list: vec![v5::CertChoice::FullCert(vec![0x01])],
                    create_date: "20250101000000Z".into(),
                    valid_start: "20250101000000Z".into(),
                    valid_end: "20300101000000Z".into(),
                },
                picture: v5::SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x01],
                    width: 100,
                    height: 100,
                },
            },
            cert: vec![0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xEE; 32],
            time_stamp: Some(vec![0x01, 0x02, 0x03]),
        };
        let der = seal.encode_der();
        let holder = VersionParser::parse_seal_version(&der).unwrap();
        assert_eq!(holder.version(), SESVersion::V5);
    }

    #[test]
    fn parse_invalid_data_returns_error() {
        assert!(VersionParser::parse_seal_version(&[0x00]).is_err());
        assert!(VersionParser::parse_signature_version(&[0x00]).is_err());
    }
}
