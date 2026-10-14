//! SES V1 签名容器。
//!
//! 对应 Java: `org.ofdrw.sign.signContainer.SESV1Container`
//!
//! 签名流程（对应 Java SESV1Container#sign）：
//! 1. SM3 摘要 Signature.xml 原文 → dataHash
//! 2. 构建 TBS_Sign{version=1, eseal, timeInfo, dataHash, propertyInfo, cert, signatureAlgorithm}
//! 3. SM3WithSM2 签名 TBS_Sign DER → sign
//! 4. 构建 SES_Signature{toSign=TBS_Sign, signature}
//! 5. 返回 SES_Signature DER

#[cfg(test)]
use super::SM2_SM3_OID_ARCS;
use super::{ExtendSignatureContainer, SM2_SM3_OID_STR, SigType};
use crate::errors::SignError;
#[cfg(test)]
use crate::internal_helpers::compute_sm3;

/// SES V1 签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.SESV1Container`
pub struct SesV1Container {
    /// SM2 签名私钥。
    secret_key: sm2::SecretKey,
    /// 电子印章 DER 编码。
    seal_der: Vec<u8>,
    /// 签章者证书 DER 编码。
    cert_der: Vec<u8>,
}

impl SesV1Container {
    /// 创建 SES V1 签名容器。
    ///
    /// 对应 Java: `SESV1Container(PrivateKey privateKey, SESeal seal, Certificate signCert)`
    #[must_use]
    pub fn new(secret_key: sm2::SecretKey, seal_der: Vec<u8>, cert_der: Vec<u8>) -> Self {
        Self {
            secret_key,
            seal_der,
            cert_der,
        }
    }
}

impl ExtendSignatureContainer for SesV1Container {
    fn sign_alg_oid(&self) -> &str {
        SM2_SM3_OID_STR
    }

    /// 对待签名数据进行电子签章。
    ///
    /// 对应 Java: `org.ofdrw.sign.signContainer.SESV1Container#sign`
    fn sign(&self, in_data: &[u8], property_info: &str) -> Result<Vec<u8>, SignError> {
        crate::ses_signing::sign(
            1,
            &self.secret_key,
            &self.seal_der,
            &self.cert_der,
            in_data,
            property_info,
            None,
        )
    }

    fn seal(&self) -> Option<Vec<u8>> {
        Some(self.seal_der.clone())
    }

    fn sign_type(&self) -> SigType {
        SigType::Seal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_gm::ses::v1::*;
    use sm2::elliptic_curve::Generate;

    fn build_test_seal_der() -> Vec<u8> {
        let seal = SESeal {
            eseal_info: SealInfo {
                header: SESHeader {
                    id: "ES".into(),
                    version: 1,
                    vid: "http://test.ofdrw.org".into(),
                },
                es_id: "ES_V1_001".into(),
                property: SESPropertyInfo {
                    seal_type: 0,
                    name: "TestSeal".into(),
                    cert_list: vec![vec![0x30, 0x03, 0x02, 0x01, 0x01]],
                    create_date: "250101000000Z".into(),
                    valid_start: "250101000000Z".into(),
                    valid_end: "300101000000Z".into(),
                },
                picture: SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x89, 0x50, 0x4E, 0x47],
                    width: 200,
                    height: 200,
                },
            },
            sign_info: SignInfo {
                cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
                signature_algorithm: SM2_SM3_OID_ARCS.to_vec(),
                sign_data: vec![0xAA; 64],
            },
        };
        seal.encode_der()
    }

    #[test]
    fn ses_v1_sign_returns_non_empty() {
        let c = SesV1Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        );
        let result = c.sign(b"test data", "prop").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn ses_v1_sign_produces_valid_der() {
        let cert_der = vec![0x30, 0x03, 0x02, 0x01, 0x01];
        let c = SesV1Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            cert_der.clone(),
        );
        let result = c.sign(b"test data", "prop").unwrap();
        let ses_sig = SESSignature::decode_der(&result).expect("V1 SES_Signature DER 解码失败");
        assert_eq!(ses_sig.to_sign.version, 1);
        assert_eq!(ses_sig.to_sign.cert, cert_der);
        assert!(!ses_sig.sign_data.is_empty());
    }

    #[test]
    fn ses_v1_sign_data_hash_matches_sm3() {
        let c = SesV1Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        );
        let in_data = b"signature xml content";
        let result = c.sign(in_data, "prop").unwrap();
        let ses_sig = SESSignature::decode_der(&result).unwrap();
        let expected_hash = compute_sm3(in_data);
        assert_eq!(ses_sig.to_sign.data_hash, expected_hash.to_vec());
    }
}
