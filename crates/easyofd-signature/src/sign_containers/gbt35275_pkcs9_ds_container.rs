//! GB/T 35275 PKCS#9 数字签名容器。
//!
//! 对应 Java: `org.ofdrw.sign.signContainer.GBT35275PKCS9DSContainer`
//!
//! 与 GBT35275DSContainer 的区别：签名对象为 PKCS#9 authenticatedAttributes
//! （含签名时间、原文杂凑值），而非直接的杂凑值。

use super::{
    ExtendSignatureContainer, SM2_SM3_OID_STR, SigType, generalized_time_now, sm2_sign_with_sm3,
};
use crate::errors::SignError;
use crate::internal_helpers::compute_sm3;
use easyofd_gm::sm2_struct::ContentInfo;
use easyofd_gm::sm2_struct::IssuerAndSerialNumber;
use easyofd_gm::sm2_struct::SignerInfo;
use easyofd_gm::sm2_struct::builder::SignedDataBuilder;
use easyofd_gm::sm2_struct::oids::{DATA, SM2_SIGN, SM3, parse_oid};

/// GB/T 35275 PKCS#9 数字签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.GBT35275PKCS9DSContainer`
pub struct Gbt35275Pkcs9DsContainer {
    /// SM2 签名私钥。
    secret_key: sm2::SecretKey,
    /// 签章者证书 DER 编码。
    cert_der: Vec<u8>,
}

impl Gbt35275Pkcs9DsContainer {
    /// 创建 GB/T 35275 PKCS#9 签名容器。
    ///
    /// 对应 Java: `GBT35275PKCS9DSContainer(Certificate cert, PrivateKey prvKey)`
    #[must_use]
    pub fn new(secret_key: sm2::SecretKey, cert_der: Vec<u8>) -> Self {
        Self {
            secret_key,
            cert_der,
        }
    }
}

impl ExtendSignatureContainer for Gbt35275Pkcs9DsContainer {
    fn sign_alg_oid(&self) -> &str {
        SM2_SM3_OID_STR
    }

    /// 对待签名数据签名。
    ///
    /// 对应 Java: `org.ofdrw.sign.signContainer.GBT35275PKCS9DSContainer#sign`
    fn sign(&self, in_data: &[u8], _property_info: &str) -> Result<Vec<u8>, SignError> {
        // 1. SM3 摘要
        let plaintext = compute_sm3(in_data);

        // 2. 构建 PKCS#9 认证属性（signingTime + messageDigest）
        let auth_attrs = build_pkcs9_auth_attrs(&plaintext);

        // 3. SM3WithSM2 签名认证属性
        let signature = sm2_sign_with_sm3(&self.secret_key, &auth_attrs)?;

        // 4. 构建内层 ContentInfo（data 类型）
        let mut content_bytes = Vec::new();
        easyofd_gm::ses::encode_octet_string(&plaintext, &mut content_bytes);
        let data_ci = ContentInfo::new(parse_oid(DATA), content_bytes);

        // 5. 从证书 DER 提取真实的颁发者和序列号，构建 SignerInfo（含认证属性）。
        //    对应 Java: `new IssuerAndSerialNumber(cert.getIssuerX500Principal(), cert.getSerialNumber())`
        let issuer_serial = IssuerAndSerialNumber::try_from_certificate_der(&self.cert_der)
            .map_err(|e| SignError::CertificateParse(e.to_string()))?;
        let signer = SignerInfo {
            version: 1,
            issuer_serial,
            digest_algorithm: parse_oid(SM3),
            authenticated_attributes: auth_attrs,
            digest_encryption_algorithm: parse_oid(SM2_SIGN),
            encrypted_digest: signature,
            unauthenticated_attributes: Vec::new(),
        };

        // 6. 构建 SignedData
        let signed_data = SignedDataBuilder::new()
            .content_info(data_ci)
            .add_certificate(self.cert_der.clone())
            .add_signer(signer)
            .build();

        // 7. 包装为外层 ContentInfo
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

/// 构建 PKCS#9 认证属性（signingTime + messageDigest）。
///
/// 对应 Java: `PKCS9SignedDataBuilder` 内部的 authenticatedAttributes 构建。
///
/// 返回 SET OF Attribute 的 DER 字节（含 0x31 SET 标签）。
fn build_pkcs9_auth_attrs(sm3_hash: &[u8]) -> Vec<u8> {
    let mut attrs = Vec::new();

    // signingTime: OID 1.2.840.113549.1.9.5
    let time_oid: [u32; 7] = [1, 2, 840, 113_549, 1, 9, 5];
    let now = generalized_time_now();
    let mut time_attr = Vec::new();
    easyofd_gm::ses::encode_oid(&time_oid, &mut time_attr);
    let mut time_val_set = Vec::new();
    let mut time_der = Vec::new();
    time_der.push(0x18); // GeneralizedTime tag
    easyofd_gm::ses::encode_length(now.len(), &mut time_der);
    time_der.extend_from_slice(now.as_bytes());
    time_val_set.push(0x31); // SET tag
    easyofd_gm::ses::encode_length(time_der.len(), &mut time_val_set);
    time_val_set.extend_from_slice(&time_der);
    time_attr.extend_from_slice(&time_val_set);
    // 包装为 SEQUENCE
    let mut time_seq = Vec::new();
    easyofd_gm::ses::encode_sequence(&time_attr, &mut time_seq);
    attrs.extend_from_slice(&time_seq);

    // messageDigest: OID 1.2.840.113549.1.9.4
    let md_oid: [u32; 7] = [1, 2, 840, 113_549, 1, 9, 4];
    let mut md_attr = Vec::new();
    easyofd_gm::ses::encode_oid(&md_oid, &mut md_attr);
    let mut md_val_set = Vec::new();
    let mut md_der = Vec::new();
    easyofd_gm::ses::encode_octet_string(sm3_hash, &mut md_der);
    md_val_set.push(0x31); // SET tag
    easyofd_gm::ses::encode_length(md_der.len(), &mut md_val_set);
    md_val_set.extend_from_slice(&md_der);
    md_attr.extend_from_slice(&md_val_set);
    let mut md_seq = Vec::new();
    easyofd_gm::ses::encode_sequence(&md_attr, &mut md_seq);
    attrs.extend_from_slice(&md_seq);

    // 包装为 SET OF
    let mut set = Vec::new();
    set.push(0x31); // SET tag
    easyofd_gm::ses::encode_length(attrs.len(), &mut set);
    set.extend_from_slice(&attrs);
    set
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
    fn gbt35275_pkcs9_sign_returns_non_empty() {
        let (sk, cert_der) = generate_key_and_cert();
        let c = Gbt35275Pkcs9DsContainer::new(sk, cert_der);
        let result = c.sign(b"test data", "").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn gbt35275_pkcs9_sign_produces_valid_content_info() {
        let (sk, cert_der) = generate_key_and_cert();
        let c = Gbt35275Pkcs9DsContainer::new(sk, cert_der);
        let result = c.sign(b"test data", "").unwrap();
        let ci = ContentInfo::from_der(&result).expect("ContentInfo DER 解码失败");
        assert_eq!(
            ci.content_type,
            parse_oid(easyofd_gm::sm2_struct::oids::SIGNED_DATA)
        );
    }

    #[test]
    fn gbt35275_pkcs9_sign_issuer_serial_matches_cert() {
        let (sk, cert_der) = generate_key_and_cert();
        let expected_iasn = IssuerAndSerialNumber::from_certificate_der(&cert_der);

        let c = Gbt35275Pkcs9DsContainer::new(sk, cert_der.clone());
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
    fn gbt35275_pkcs9_properties() {
        let sk = sm2::SecretKey::generate();
        let cert_der = vec![0x01]; // properties 不触发 sign，不需要真实证书
        let c = Gbt35275Pkcs9DsContainer::new(sk, cert_der);
        assert_eq!(c.sign_alg_oid(), "1.2.156.10197.1.501");
        assert_eq!(c.sign_type(), SigType::Sign);
        assert!(c.seal().is_none());
    }

    #[test]
    fn build_pkcs9_auth_attrs_non_empty() {
        let attrs = build_pkcs9_auth_attrs(&[0xAA; 32]);
        assert!(!attrs.is_empty());
        assert_eq!(attrs[0], 0x31); // SET tag
    }
}
