//! GB/T 35275 数字签名容器。
//!
//! 对应 Java: `org.ofdrw.sign.signContainer.GBT35275DSContainer`
//!
//! 签名流程（对应 Java GBT35275DSContainer#sign）：
//! 1. SM3 摘要 inData → plaintext
//! 2. SM3WithSM2 签名 plaintext → signature
//! 3. 构建 CMS SignedData(ContentInfo(data, plaintext), cert, SignerInfo)
//! 4. 包装为 ContentInfo(signedData, SignedData)
//! 5. 返回 ContentInfo DER

use super::{ExtendSignatureContainer, SM2_SM3_OID_STR, SigType, sm2_sign_with_sm3};
use crate::errors::SignError;
use crate::internal_helpers::compute_sm3;
use easyofd_gm::sm2_struct::ContentInfo;
use easyofd_gm::sm2_struct::IssuerAndSerialNumber;
use easyofd_gm::sm2_struct::SignerInfo;
use easyofd_gm::sm2_struct::builder::SignedDataBuilder;
use easyofd_gm::sm2_struct::oids::{DATA, SM2_SIGN, SM3, parse_oid};

/// GB/T 35275 数字签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.GBT35275DSContainer`
pub struct Gbt35275DsContainer {
    /// SM2 签名私钥。
    secret_key: sm2::SecretKey,
    /// 签章者证书 DER 编码。
    cert_der: Vec<u8>,
}

impl Gbt35275DsContainer {
    /// 创建 GB/T 35275 签名容器。
    ///
    /// 对应 Java: `GBT35275DSContainer(Certificate cert, PrivateKey prvKey)`
    #[must_use]
    pub fn new(secret_key: sm2::SecretKey, cert_der: Vec<u8>) -> Self {
        Self {
            secret_key,
            cert_der,
        }
    }
}

impl ExtendSignatureContainer for Gbt35275DsContainer {
    fn sign_alg_oid(&self) -> &str {
        SM2_SM3_OID_STR
    }

    /// 对待签名数据签名。
    ///
    /// 对应 Java: `org.ofdrw.sign.signContainer.GBT35275DSContainer#sign`
    fn sign(&self, in_data: &[u8], _property_info: &str) -> Result<Vec<u8>, SignError> {
        // 1. SM3 摘要
        let plaintext = compute_sm3(in_data);

        // 2. SM3WithSM2 签名摘要值
        let signature = sm2_sign_with_sm3(&self.secret_key, &plaintext)?;

        // 3. 构建内层 ContentInfo（data 类型，内容为摘要值的 OCTET STRING 编码）
        let mut content_bytes = Vec::new();
        easyofd_gm::ses::encode_octet_string(&plaintext, &mut content_bytes);
        let data_ci = ContentInfo::new(parse_oid(DATA), content_bytes);

        // 4. 从证书 DER 提取真实的颁发者和序列号，构造 SignerInfo。
        //    对应 Java: `new IssuerAndSerialNumber(cert.getIssuerX500Principal(), cert.getSerialNumber())`
        let issuer_serial = IssuerAndSerialNumber::try_from_certificate_der(&self.cert_der)
            .map_err(|e| SignError::CertificateParse(e.to_string()))?;
        let signer = SignerInfo::new(
            issuer_serial,
            parse_oid(SM3),
            parse_oid(SM2_SIGN),
            signature,
        );

        // 5. 构建 SignedData
        let signed_data = SignedDataBuilder::new()
            .content_info(data_ci)
            .add_certificate(self.cert_der.clone())
            .add_signer(signer)
            .build();

        // 6. 包装为外层 ContentInfo
        let outer_ci = ContentInfo::from_signed_data(&signed_data)
            .map_err(|e| SignError::Encode(format!("ContentInfo 构建失败: {e}")))?;
        outer_ci
            .to_der()
            .map_err(|e| SignError::Encode(format!("ContentInfo DER 编码失败: {e}")))
    }

    fn seal(&self) -> Option<Vec<u8>> {
        None
    }

    fn sign_type(&self) -> SigType {
        SigType::Sign
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_gm::pkc_generate::PkcGenerate;
    use easyofd_gm::sm2_struct::SignedData;
    use sm2::elliptic_curve::Generate;

    /// 生成 SM2 密钥对和对应的自签名证书 DER。
    fn generate_key_and_cert() -> (sm2::SecretKey, Vec<u8>) {
        let sk = sm2::SecretKey::generate();
        let cert_der = PkcGenerate::generate_self_signed("CN=Test Signer,O=TestOrg,C=CN");
        (sk, cert_der)
    }

    #[test]
    fn gbt35275_ds_sign_returns_non_empty() {
        let (sk, cert_der) = generate_key_and_cert();
        let c = Gbt35275DsContainer::new(sk, cert_der);
        let result = c.sign(b"test data", "").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn gbt35275_ds_sign_produces_valid_content_info() {
        let (sk, cert_der) = generate_key_and_cert();
        let c = Gbt35275DsContainer::new(sk, cert_der);
        let result = c.sign(b"test data", "").unwrap();
        let ci = ContentInfo::from_der(&result).expect("ContentInfo DER 解码失败");
        // contentType 应为 signedData OID
        assert_eq!(
            ci.content_type,
            parse_oid(easyofd_gm::sm2_struct::oids::SIGNED_DATA)
        );
    }

    #[test]
    fn gbt35275_ds_sign_issuer_serial_matches_cert() {
        let (sk, cert_der) = generate_key_and_cert();
        let expected_iasn = IssuerAndSerialNumber::from_certificate_der(&cert_der);

        let c = Gbt35275DsContainer::new(sk, cert_der.clone());
        let result = c.sign(b"test data", "").unwrap();

        // 解码 ContentInfo → SignedData → 提取 SignerInfo 的 issuer/serial。
        let ci = ContentInfo::from_der(&result).expect("ContentInfo DER 解码失败");
        let sd = SignedData::from_der(&ci.content).expect("SignedData DER 解码失败");
        assert_eq!(sd.signer_infos.len(), 1, "应有且仅有一个签名者");

        let actual_iasn = &sd.signer_infos[0].issuer_serial;
        assert_eq!(
            actual_iasn.issuer_der, expected_iasn.issuer_der,
            "SignerInfo 的 issuer 应与证书一致"
        );
        assert_eq!(
            actual_iasn.cert_serial_number, expected_iasn.cert_serial_number,
            "SignerInfo 的 serial number 应与证书一致"
        );
    }

    #[test]
    fn gbt35275_ds_properties() {
        let sk = sm2::SecretKey::generate();
        let cert_der = vec![0x01]; // properties 不触发 sign，不需要真实证书
        let c = Gbt35275DsContainer::new(sk, cert_der);
        assert_eq!(c.sign_alg_oid(), "1.2.156.10197.1.501");
        assert_eq!(c.sign_type(), SigType::Sign);
        assert!(c.seal().is_none());
    }
}
