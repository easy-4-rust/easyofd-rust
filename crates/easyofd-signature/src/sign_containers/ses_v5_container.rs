//! SES V5 签名容器。
//!
//! 对应 Java: `org.ofdrw.sign.signContainer.SESV5Container`
//!
//! V5 与 V4 流程相同，区别在于印章 SESeal 含可选 timeStamp。

use super::{
    ExtendSignatureContainer, SM2_SM3_OID_ARCS, SM2_SM3_OID_STR, SigType, generalized_time_now,
    sm2_sign_with_sm3,
};
use crate::internal_helpers::compute_sm3;

/// SES V5 签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.SESV5Container`
pub struct SesV5Container {
    /// SM2 签名私钥。
    secret_key: sm2::SecretKey,
    /// 电子印章 DER 编码。
    seal_der: Vec<u8>,
    /// 签章者证书 DER 编码。
    cert_der: Vec<u8>,
}

impl SesV5Container {
    /// 创建 SES V5 签名容器。
    ///
    /// 对应 Java: `SESV5Container(PrivateKey privateKey, SESeal seal, Certificate signCert)`
    #[must_use]
    pub fn new(secret_key: sm2::SecretKey, seal_der: Vec<u8>, cert_der: Vec<u8>) -> Self {
        Self {
            secret_key,
            seal_der,
            cert_der,
        }
    }
}

impl ExtendSignatureContainer for SesV5Container {
    fn sign_alg_oid(&self) -> &str {
        SM2_SM3_OID_STR
    }

    /// 对待签名数据进行电子签章。
    ///
    /// 对应 Java: `org.ofdrw.sign.signContainer.SESV5Container#sign`
    fn sign(&self, in_data: &[u8], property_info: &str) -> Vec<u8> {
        let data_hash = compute_sm3(in_data);

        let seal =
            easyofd_gm::ses::v5::SESeal::decode_der(&self.seal_der).expect("V5 印章 DER 解码失败");

        let tbs = easyofd_gm::ses::v5::TBSSign {
            version: 5,
            seal,
            time_info: generalized_time_now(),
            data_hash: data_hash.to_vec(),
            property_info: property_info.to_string(),
        };
        let tbs_der = tbs.encode_der();

        let sig_val = sm2_sign_with_sm3(&self.secret_key, &tbs_der);

        let ses_sig = easyofd_gm::ses::v5::SESSignature {
            to_sign: tbs,
            cert: self.cert_der.clone(),
            signature_algorithm: SM2_SM3_OID_ARCS.to_vec(),
            sign_data: sig_val,
        };

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
    use easyofd_gm::ses::v5::*;
    use sm2::elliptic_curve::Generate;

    fn build_test_seal_der() -> Vec<u8> {
        let seal = SESeal {
            eseal_info: SealInfo {
                header: SESHeader {
                    id: "ES".into(),
                    version: 5,
                    vid: "http://test.ofdrw.org".into(),
                },
                es_id: "ES_V5_001".into(),
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
            time_stamp: None,
        };
        seal.encode_der()
    }

    #[test]
    fn ses_v5_sign_returns_non_empty() {
        let c = SesV5Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        );
        let result = c.sign(b"test data", "prop");
        assert!(!result.is_empty());
    }

    #[test]
    fn ses_v5_sign_produces_valid_der() {
        let cert_der = vec![0x30, 0x03, 0x02, 0x01, 0x01];
        let c = SesV5Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            cert_der.clone(),
        );
        let result = c.sign(b"test data", "prop");
        let ses_sig = SESSignature::decode_der(&result).expect("V5 SES_Signature DER 解码失败");
        assert_eq!(ses_sig.to_sign.version, 5);
        assert_eq!(ses_sig.to_sign.property_info, "prop");
        assert!(!ses_sig.sign_data.is_empty());
    }

    #[test]
    fn ses_v5_sign_data_hash_matches_sm3() {
        let c = SesV5Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        );
        let in_data = b"signature xml content";
        let result = c.sign(in_data, "prop");
        let ses_sig = SESSignature::decode_der(&result).unwrap();
        let expected_hash = compute_sm3(in_data);
        assert_eq!(ses_sig.to_sign.data_hash, expected_hash.to_vec());
    }
}
