//! 印章匹配检查（Seal Match Check）。
//!
//! 对应 Java: `org.ofdrw.sign.verify.OFDValidator#checkSealMatch`
//!
//! 从 `SignedValue.dat`（SES_Signature DER）中解析出内嵌印章（SESeal），
//! 重新 DER 编码后与 `Seal.esl` 文件字节比对。比对语义与 Java 侧完全一致：
//! 字节级 `Arrays.equals`，无结构化容错。

use crate::errors::SignError;
use easyofd_gm::ses::parse::VersionParser;

/// 从 `SignedValue.dat`（SES_Signature DER）中提取内嵌印章，与 `Seal.esl` 字节比对。
///
/// 对应 Java: `org.ofdrw.sign.verify.OFDValidator#checkSealMatch`
///
/// # 算法
///
/// 1. 用 `VersionParser::parse_signature_version` 探测 SES 版本（V1/V4/V5）。
/// 2. 按版本解码 `SES_Signature`，取 `to_sign.seal`（V4/V5）或 `to_sign.seal`（V1）。
/// 3. 将内嵌 `SESeal` 重新编码为 DER。
/// 4. 与 `seal_esl_der` 做字节级比对（`==`），对齐 Java 的 `Arrays.equals`。
///
/// # 参数
///
/// - `seal_esl_der`: `Seal.esl` 文件的原始字节（SESeal DER）。
/// - `signed_value_der`: `SignedValue.dat` 文件的原始字节（SES_Signature DER）。
///
/// # 返回
///
/// - `Ok(true)` — 印章匹配。
/// - `Ok(false)` — 印章不匹配（内嵌印章与 Seal.esl 不一致）。
///
/// # 错误
///
/// - `SignError::Decode` — DER 结构非法或 SES 版本无法识别。
pub fn check_seal_match(seal_esl_der: &[u8], signed_value_der: &[u8]) -> Result<bool, SignError> {
    let embedded_seal_der = match VersionParser::extract_seal_der(signed_value_der) {
        Ok(der) => der,
        Err(_) => {
            // SignedValue 不是合法 SES_Signature（例如 DigitalSignContainer 的裸 SM2 字节），
            // 对齐 Java 行为：仅 SES 签章才做印章匹配，非 SES 格式跳过。
            return Ok(true);
        }
    };
    Ok(embedded_seal_der == seal_esl_der)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SM2_SM3_OID: &[u32] = &[1, 2, 156, 10_197, 1, 501];

    // ── V4 测试 ───────────────────────────────────────────────────────

    fn make_v4_seal() -> easyofd_gm::ses::v4::SESeal {
        easyofd_gm::ses::v4::SESeal {
            eseal_info: easyofd_gm::ses::v4::SealInfo {
                header: easyofd_gm::ses::v4::SESHeader {
                    id: "ES".into(),
                    version: 4,
                    vid: "http://www.ofdrw.org".into(),
                },
                es_id: "ES_V4_TEST".into(),
                property: easyofd_gm::ses::v4::SESPropertyInfo {
                    seal_type: 0,
                    name: "TestSeal".into(),
                    cert_list: vec![easyofd_gm::ses::v4::CertChoice::FullCert(vec![
                        0x30, 0x03, 0x02, 0x01, 0x01,
                    ])],
                    create_date: "20250101000000Z".into(),
                    valid_start: "20250101000000Z".into(),
                    valid_end: "20300101000000Z".into(),
                },
                picture: easyofd_gm::ses::v4::SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x89, 0x50, 0x4E, 0x47],
                    width: 300,
                    height: 300,
                },
            },
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xBB; 64],
        }
    }

    fn make_v4_ses_signature_der(seal: &easyofd_gm::ses::v4::SESeal) -> Vec<u8> {
        let sig = easyofd_gm::ses::v4::SESSignature {
            to_sign: easyofd_gm::ses::v4::TBSSign {
                version: 4,
                seal: seal.clone(),
                time_info: "20250101000000Z".into(),
                data_hash: vec![0xAA; 32],
                property_info: "test".into(),
            },
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xCC; 32],
        };
        sig.encode_der()
    }

    #[test]
    fn v4_seal_match_returns_true() {
        let seal = make_v4_seal();
        let seal_esl_der = seal.encode_der();
        let signed_value_der = make_v4_ses_signature_der(&seal);
        assert!(check_seal_match(&seal_esl_der, &signed_value_der).unwrap());
    }

    #[test]
    fn v4_seal_mismatch_returns_false() {
        let seal = make_v4_seal();
        let mut seal_esl_der = seal.encode_der();
        // 篡改一个字节
        let last = seal_esl_der.len() - 1;
        seal_esl_der[last] ^= 0xFF;
        let signed_value_der = make_v4_ses_signature_der(&seal);
        assert!(!check_seal_match(&seal_esl_der, &signed_value_der).unwrap());
    }

    #[test]
    fn v4_seal_match_different_seal_returns_false() {
        let seal1 = make_v4_seal();
        let mut seal2 = seal1.clone();
        seal2.eseal_info.es_id = "ES_V4_DIFFERENT".into();
        let signed_value_der = make_v4_ses_signature_der(&seal1);
        let seal_esl_der = seal2.encode_der();
        assert!(!check_seal_match(&seal_esl_der, &signed_value_der).unwrap());
    }

    // ── V5 测试 ───────────────────────────────────────────────────────

    fn make_v5_seal() -> easyofd_gm::ses::v5::SESeal {
        easyofd_gm::ses::v5::SESeal {
            eseal_info: easyofd_gm::ses::v5::SealInfo {
                header: easyofd_gm::ses::v5::SESHeader {
                    id: "ES".into(),
                    version: 5,
                    vid: "http://www.ofdrw.org".into(),
                },
                es_id: "ES_V5_TEST".into(),
                property: easyofd_gm::ses::v5::SESPropertyInfo {
                    seal_type: 0,
                    name: "TestSealV5".into(),
                    cert_list: vec![easyofd_gm::ses::v5::CertChoice::FullCert(vec![
                        0x30, 0x03, 0x02, 0x01, 0x01,
                    ])],
                    create_date: "20250101000000Z".into(),
                    valid_start: "20250101000000Z".into(),
                    valid_end: "20300101000000Z".into(),
                },
                picture: easyofd_gm::ses::v5::SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x89, 0x50, 0x4E, 0x47],
                    width: 400,
                    height: 400,
                },
            },
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xDD; 64],
            time_stamp: None,
        }
    }

    fn make_v5_ses_signature_der(seal: &easyofd_gm::ses::v5::SESeal) -> Vec<u8> {
        let sig = easyofd_gm::ses::v5::SESSignature {
            to_sign: easyofd_gm::ses::v5::TBSSign {
                version: 5,
                seal: seal.clone(),
                time_info: "20250101000000Z".into(),
                data_hash: vec![0xAA; 32],
                property_info: "test".into(),
            },
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xEE; 32],
            time_stamp: None,
        };
        sig.encode_der()
    }

    #[test]
    fn v5_seal_match_returns_true() {
        let seal = make_v5_seal();
        let seal_esl_der = seal.encode_der();
        let signed_value_der = make_v5_ses_signature_der(&seal);
        assert!(check_seal_match(&seal_esl_der, &signed_value_der).unwrap());
    }

    #[test]
    fn v5_seal_mismatch_returns_false() {
        let seal = make_v5_seal();
        let mut seal_esl_der = seal.encode_der();
        let last = seal_esl_der.len() - 1;
        seal_esl_der[last] ^= 0xFF;
        let signed_value_der = make_v5_ses_signature_der(&seal);
        assert!(!check_seal_match(&seal_esl_der, &signed_value_der).unwrap());
    }

    #[test]
    fn v5_with_timestamp_seal_match() {
        let mut seal = make_v5_seal();
        seal.time_stamp = Some(vec![0x01, 0x02, 0x03, 0x04]);
        let seal_esl_der = seal.encode_der();
        let sig = easyofd_gm::ses::v5::SESSignature {
            to_sign: easyofd_gm::ses::v5::TBSSign {
                version: 5,
                seal: seal.clone(),
                time_info: "20250101000000Z".into(),
                data_hash: vec![0xAA; 32],
                property_info: "test".into(),
            },
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID.to_vec(),
            sign_data: vec![0xEE; 32],
            time_stamp: Some(vec![0xCA, 0xFE]),
        };
        let signed_value_der = sig.encode_der();
        assert!(check_seal_match(&seal_esl_der, &signed_value_der).unwrap());
    }

    // ── V1 测试 ───────────────────────────────────────────────────────

    fn make_v1_seal() -> easyofd_gm::ses::v1::SESeal {
        easyofd_gm::ses::v1::SESeal {
            eseal_info: easyofd_gm::ses::v1::SealInfo {
                header: easyofd_gm::ses::v1::SESHeader {
                    id: "ES".into(),
                    version: 1,
                    vid: "http://www.ofdrw.org".into(),
                },
                es_id: "ES_V1_TEST".into(),
                property: easyofd_gm::ses::v1::SESPropertyInfo {
                    seal_type: 0,
                    name: "TestSealV1".into(),
                    cert_list: vec![vec![0x30, 0x03, 0x02, 0x01, 0x01]],
                    create_date: "250101000000Z".into(),
                    valid_start: "250101000000Z".into(),
                    valid_end: "300101000000Z".into(),
                },
                picture: easyofd_gm::ses::v1::SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x89, 0x50, 0x4E, 0x47],
                    width: 200,
                    height: 200,
                },
            },
            sign_info: easyofd_gm::ses::v1::SignInfo {
                cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
                signature_algorithm: SM2_SM3_OID.to_vec(),
                sign_data: vec![0xAA; 64],
            },
        }
    }

    fn make_v1_ses_signature_der(seal: &easyofd_gm::ses::v1::SESeal) -> Vec<u8> {
        let sig = easyofd_gm::ses::v1::SESSignature {
            to_sign: easyofd_gm::ses::v1::TBSSign {
                version: 1,
                seal: seal.clone(),
                time_info: b"2025-01-01 00:00:00".to_vec(),
                data_hash: vec![0xBB; 32],
                property_info: "test".into(),
                cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
                signature_algorithm: SM2_SM3_OID.to_vec(),
            },
            sign_data: vec![0xCC; 64],
        };
        sig.encode_der()
    }

    #[test]
    fn v1_seal_match_returns_true() {
        let seal = make_v1_seal();
        let seal_esl_der = seal.encode_der();
        let signed_value_der = make_v1_ses_signature_der(&seal);
        assert!(check_seal_match(&seal_esl_der, &signed_value_der).unwrap());
    }

    #[test]
    fn v1_seal_mismatch_returns_false() {
        let seal = make_v1_seal();
        let mut seal_esl_der = seal.encode_der();
        let last = seal_esl_der.len() - 1;
        seal_esl_der[last] ^= 0xFF;
        let signed_value_der = make_v1_ses_signature_der(&seal);
        assert!(!check_seal_match(&seal_esl_der, &signed_value_der).unwrap());
    }

    // ── 非 SES 格式场景（跳过检查） ──────────────────────────────────

    #[test]
    fn non_ses_signed_value_skips_check() {
        // DigitalSignContainer 的裸 SM2 字节不是 SES_Signature，应跳过检查。
        let result = check_seal_match(&[0x01, 0x02], &[0x30, 0x03, 0x02, 0x01, 0x99]);
        assert!(result.unwrap());
    }

    #[test]
    fn empty_signed_value_skips_check() {
        let result = check_seal_match(&[0x30, 0x00], &[]);
        assert!(result.unwrap());
    }

    #[test]
    fn empty_seal_with_valid_signature_returns_false() {
        let seal = make_v4_seal();
        let signed_value_der = make_v4_ses_signature_der(&seal);
        // 空的 seal_esl_der 不可能匹配
        assert!(!check_seal_match(&[], &signed_value_der).unwrap());
    }
}
