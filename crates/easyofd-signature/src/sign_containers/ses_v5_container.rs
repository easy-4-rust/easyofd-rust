//! SES V5 签名容器。
//!
//! 对应 Java: `org.ofdrw.sign.signContainer.SESV5Container`
//!
//! V5 与 V4 流程相同，区别在于印章 SESeal 含可选 timeStamp。

#[cfg(test)]
use super::SM2_SM3_OID_ARCS;
use super::{ExtendSignatureContainer, SM2_SM3_OID_STR, SigType};
use crate::errors::SignError;
#[cfg(test)]
use crate::internal_helpers::compute_sm3;
use crate::timestamp_hook::TimeStampHook;

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
    /// 可选时间戳 Hook，在签名完成后获取签名值的时间戳。
    ///
    /// 对应 Java: `SESV5Container#timeStampHook`
    time_stamp_hook: Option<Box<dyn TimeStampHook>>,
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
            time_stamp_hook: None,
        }
    }

    /// 设置时间戳 Hook。
    ///
    /// 对应 Java: `SESV5Container#setTimeStampHook(TimeStampHook)`
    ///
    /// 在签名完成后调用 `hook.apply(sig_val)` 获取签名值的可信时间戳。
    #[must_use]
    pub fn with_time_stamp_hook(mut self, hook: impl TimeStampHook + 'static) -> Self {
        self.time_stamp_hook = Some(Box::new(hook));
        self
    }
}

impl ExtendSignatureContainer for SesV5Container {
    fn sign_alg_oid(&self) -> &str {
        SM2_SM3_OID_STR
    }

    /// 对待签名数据进行电子签章。
    ///
    /// 对应 Java: `org.ofdrw.sign.signContainer.SESV5Container#sign`
    fn sign(&self, in_data: &[u8], property_info: &str) -> Result<Vec<u8>, SignError> {
        crate::ses_signing::sign(
            5,
            &self.secret_key,
            &self.seal_der,
            &self.cert_der,
            in_data,
            property_info,
            self.time_stamp_hook.as_deref(),
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
    use crate::timestamp_hook::ClosureTimeStampHook;
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
        let result = c.sign(b"test data", "prop").unwrap();
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
        let result = c.sign(b"test data", "prop").unwrap();
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
        let result = c.sign(in_data, "prop").unwrap();
        let ses_sig = SESSignature::decode_der(&result).unwrap();
        let expected_hash = compute_sm3(in_data);
        assert_eq!(ses_sig.to_sign.data_hash, expected_hash.to_vec());
    }

    /// 无 hook 时 time_stamp 为 None（DER 解码验证）。
    #[test]
    fn ses_v5_sign_no_hook_time_stamp_is_none() {
        let c = SesV5Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        );
        let result = c.sign(b"test data", "prop").unwrap();
        let ses_sig = SESSignature::decode_der(&result).unwrap();
        assert!(
            ses_sig.time_stamp.is_none(),
            "无 hook 时 time_stamp 应为 None"
        );
    }

    /// 有 hook（返回固定字节）时 SESSignature 解码出 time_stamp 等于 hook 输出。
    #[test]
    fn ses_v5_sign_with_hook_sets_time_stamp() {
        let expected_ts = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE];
        let hook = ClosureTimeStampHook::new({
            let ts = expected_ts.clone();
            move |_sig: &[u8]| ts.clone()
        });
        let c = SesV5Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        )
        .with_time_stamp_hook(hook);
        let result = c.sign(b"test data", "prop").unwrap();
        let ses_sig = SESSignature::decode_der(&result).unwrap();
        assert_eq!(
            ses_sig.time_stamp,
            Some(expected_ts),
            "有 hook 时 time_stamp 应等于 hook 输出"
        );
    }

    /// hook 返回空 Vec 时不设置 time_stamp（对齐 Java 的 null 检查）。
    #[test]
    fn ses_v5_sign_hook_returns_empty_vec_no_time_stamp() {
        let hook = ClosureTimeStampHook::new(|_sig: &[u8]| Vec::new());
        let c = SesV5Container::new(
            sm2::SecretKey::generate(),
            build_test_seal_der(),
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
        )
        .with_time_stamp_hook(hook);
        let result = c.sign(b"test data", "prop").unwrap();
        let ses_sig = SESSignature::decode_der(&result).unwrap();
        assert!(
            ses_sig.time_stamp.is_none(),
            "hook 返回空 Vec 时 time_stamp 应为 None"
        );
    }
}
