//! SES V4 签名容器。
//!
//! 对应 Java: `org.ofdrw.sign.signContainer.SESV4Container`
//!
//! 签名流程（对应 Java SESV4Container#sign）：
//! 1. SM3 摘要 Signature.xml 原文 → dataHash
//! 2. 构建 TBS_Sign{version=4, eseal, timeInfo(GeneralizedTime), dataHash, propertyInfo}
//! 3. SM3WithSM2 签名 TBS_Sign DER → sigVal
//! 4. 构建 SES_Signature{toSign=TBS_Sign, cert, signatureAlgorithm, signature}
//! 5. 返回 SES_Signature DER

use super::{
    ExtendSignatureContainer, SM2_SM3_OID_ARCS, SM2_SM3_OID_STR, SigType, generalized_time_now,
    sm2_sign_with_sm3,
};
use crate::internal_helpers::compute_sm3;

/// SES V4 签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.SESV4Container`
pub struct SesV4Container {
    /// SM2 签名私钥。
    secret_key: sm2::SecretKey,
    /// 电子印章 DER 编码。
    seal_der: Vec<u8>,
    /// 签章者证书 DER 编码。
    cert_der: Vec<u8>,
}

impl SesV4Container {
    /// 创建 SES V4 签名容器。
    ///
    /// 对应 Java: `SESV4Container(PrivateKey privateKey, SESeal seal, Certificate signCert)`
    ///
    /// # 参数
    ///
    /// - `secret_key`：SM2 签名私钥
    /// - `seal_der`：电子印章 DER 编码字节
    /// - `cert_der`：签章者证书 DER 编码字节
    #[must_use]
    pub fn new(secret_key: sm2::SecretKey, seal_der: Vec<u8>, cert_der: Vec<u8>) -> Self {
        Self {
            secret_key,
            seal_der,
            cert_der,
        }
    }
}

impl ExtendSignatureContainer for SesV4Container {
    fn sign_alg_oid(&self) -> &str {
        SM2_SM3_OID_STR
    }

    /// 对待签名数据进行电子签章。
    ///
    /// 对应 Java: `org.ofdrw.sign.signContainer.SESV4Container#sign`
    fn sign(&self, in_data: &[u8], property_info: &str) -> Vec<u8> {
        // 1. SM3 摘要 Signature.xml 原文
        let data_hash = compute_sm3(in_data);

        // 2. 解码印章 DER
        let seal =
            easyofd_gm::ses::v4::SESeal::decode_der(&self.seal_der).expect("V4 印章 DER 解码失败");

        // 3. 构建 TBS_Sign
        let tbs = easyofd_gm::ses::v4::TBSSign {
            version: 4,
            seal,
            time_info: generalized_time_now(),
            data_hash: data_hash.to_vec(),
            property_info: property_info.to_string(),
        };
        let tbs_der = tbs.encode_der();

        // 4. SM3WithSM2 签名 TBS_Sign DER
        let sig_val = sm2_sign_with_sm3(&self.secret_key, &tbs_der);

        // 5. 构建 SES_Signature（to_sign 保存完整 TBS_Sign）
        let ses_sig = easyofd_gm::ses::v4::SESSignature {
            to_sign: tbs,
            cert: self.cert_der.clone(),
            signature_algorithm: SM2_SM3_OID_ARCS.to_vec(),
            sign_data: sig_val,
        };

        // 6. 返回 SES_Signature DER
        ses_sig.encode_der()
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
    use sm2::elliptic_curve::Generate;

    /// 构建测试用 V4 SESeal DER。
    fn build_test_seal_der() -> Vec<u8> {
        use easyofd_gm::ses::v4::*;
        let seal = SESeal {
            eseal_info: SealInfo {
                header: SESHeader {
                    id: "ES".into(),
                    version: 4,
                    vid: "http://test.ofdrw.org".into(),
                },
                es_id: "ES_TEST_001".into(),
                property: SESPropertyInfo {
                    seal_type: 0,
                    name: "TestSeal".into(),
                    cert_list: vec![CertChoice::FullCert(vec![0x30, 0x03, 0x02, 0x01, 0x01])],
                    create_date: "20250101000000Z".into(),
                    valid_start: "20250101000000Z".into(),
                    valid_end: "20300101000000Z".into(),
                },
                picture: SESPictureInfo {
                    pic_type: "PNG".into(),
                    data: vec![0x89, 0x50, 0x4E, 0x47],
                    width: 200,
                    height: 200,
                },
            },
            cert: vec![0x30, 0x03, 0x02, 0x01, 0x01],
            signature_algorithm: SM2_SM3_OID_ARCS.to_vec(),
            sign_data: vec![0xAA; 64],
        };
        seal.encode_der()
    }

    #[test]
    fn ses_v4_container_properties() {
        let c = SesV4Container::new(sm2::SecretKey::generate(), vec![0x03], vec![0x04]);
        assert_eq!(c.sign_alg_oid(), "1.2.156.10197.1.501");
        assert_eq!(c.sign_type(), SigType::Seal);
        assert_eq!(c.seal(), Some(vec![0x03]));
    }

    #[test]
    fn ses_v4_sign_returns_non_empty_der() {
        let c = SesV4Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        );
        let result = c.sign(b"test signature xml data", "test-property");
        assert!(!result.is_empty(), "sign() 不应返回空 Vec");
    }

    #[test]
    fn ses_v4_sign_produces_valid_ses_signature_der() {
        use easyofd_gm::ses::v4::SESSignature;

        let cert_der = vec![0x30, 0x03, 0x02, 0x01, 0x01];
        let c = SesV4Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            cert_der.clone(),
        );
        let result = c.sign(b"test data", "prop");

        // 解析为 SES_Signature
        let ses_sig = SESSignature::decode_der(&result).expect("SES_Signature DER 解码失败");
        assert_eq!(ses_sig.cert, cert_der);
        assert!(!ses_sig.sign_data.is_empty(), "签名值不应为空");
        // TBS_Sign 可从 SES_Signature 中恢复
        assert_eq!(ses_sig.to_sign.version, 4);
        assert_eq!(ses_sig.to_sign.property_info, "prop");
    }

    #[test]
    fn ses_v4_sign_data_hash_matches_sm3() {
        use easyofd_gm::ses::v4::SESSignature;

        let c = SesV4Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        );
        let in_data = b"signature xml content for hash check";
        let result = c.sign(in_data, "prop");

        // 解码 SES_Signature 并检查 TBS_Sign 中的 data_hash
        let ses_sig = SESSignature::decode_der(&result).unwrap();
        let expected_hash = crate::internal_helpers::compute_sm3(in_data);
        assert_eq!(
            ses_sig.to_sign.data_hash,
            expected_hash.to_vec(),
            "TBS_Sign.data_hash 应与 SM3(in_data) 一致"
        );
    }
}
