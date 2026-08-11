//! SM2 自签名证书生成工具。
//!
//! 对应 Java: org.ofdrw.gm.cert.PKCGenerate
//!
//! 提供 SM2 密钥对生成和自签名 X.509 证书生成功能。

use std::str::FromStr;
use std::time::{Duration, SystemTime};

use der::Encode;
use der::asn1::{ObjectIdentifier, OctetString};
use sm2::elliptic_curve::Generate;
use x509_cert::ext::Extension;
use x509_cert::ext::pkix::{BasicConstraints, ExtendedKeyUsage, KeyUsage, KeyUsages};
use x509_cert::name::Name;
use x509_cert::serial_number::SerialNumber;
use x509_cert::time::{Time, Validity};
use x509_cert::{AlgorithmIdentifier, SubjectPublicKeyInfo};

/// 证书生成工具。
///
/// 对应 Java: org.ofdrw.gm.cert.PKCGenerate
///
/// 提供 SM2 密钥对和自签名证书生成功能。
pub struct PkcGenerate;

/// SM2withSM3 签名算法 OID: 1.2.156.10197.1.501
const SM2_WITH_SM3_OID: &str = "1.2.156.10197.1.501";

/// X.509 BasicConstraints 扩展 OID: 2.5.29.19
const OID_BASIC_CONSTRAINTS: &str = "2.5.29.19";

/// X.509 KeyUsage 扩展 OID: 2.5.29.15
const OID_KEY_USAGE: &str = "2.5.29.15";

/// X.509 ExtendedKeyUsage 扩展 OID: 2.5.29.37
const OID_EXTENDED_KEY_USAGE: &str = "2.5.29.37";

/// Netscape Cert Type 扩展 OID: 2.16.840.1.113730.1.1
const OID_NETSCAPE_CERT_TYPE: &str = "2.16.840.1.113730.1.1";

/// id-kp-clientAuth 扩展密钥用途 OID: 1.3.6.1.5.5.7.3.2
const OID_KP_CLIENT_AUTH: &str = "1.3.6.1.5.5.7.3.2";

impl PkcGenerate {
    /// 生成自签名 SM2 证书。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.PKCGenerate#generate
    ///
    /// 生成 SM2 密钥对，构造自签名 X.509 V3 证书。
    /// 证书有效期为当前时间前后各一年。
    /// 证书的 subject 和 issuer 相同（自签名）。
    ///
    /// # 参数
    /// - `subject`: 证书使用者的可分辨名称（DN），格式如 "CN=Test,O=Org"
    ///
    /// # 返回
    /// DER 编码的 X.509 证书字节。
    ///
    /// # Panics
    /// 如果密钥生成或签名操作失败（正常情况下不会发生）。
    #[must_use]
    pub fn generate_self_signed(subject: &str) -> Vec<u8> {
        Self::generate_self_signed_with_serial(subject, Self::default_serial())
    }

    /// 生成自签名 SM2 证书（指定序列号）。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.PKCGenerate#generate
    ///
    /// 与 `generate_self_signed` 相同，但允许指定证书序列号。
    ///
    /// # 参数
    /// - `subject`: 证书使用者的可分辨名称（DN）
    /// - `serial_number`: 证书序列号（必须为正整数，不超过 20 字节）
    ///
    /// # 返回
    /// DER 编码的 X.509 证书字节。
    ///
    /// # Panics
    /// 如果密钥生成、证书构造或签名操作失败。
    #[must_use]
    pub fn generate_self_signed_with_serial(subject: &str, serial_number: u64) -> Vec<u8> {
        // 1. 生成 SM2 密钥对
        let sk = sm2::SecretKey::generate();
        let signing_key =
            sm2::dsa::SigningKey::new("1234567812345678", &sk).expect("SM2 签名密钥构造失败");
        let vk = signing_key.verifying_key();

        // 2. 构造 SubjectPublicKeyInfo
        let spki = SubjectPublicKeyInfo::from_key(vk).expect("SubjectPublicKeyInfo 构造失败");

        // 3. 构造证书各字段
        let sig_oid = der::asn1::ObjectIdentifier::new(SM2_WITH_SM3_OID).expect("OID 解析失败");
        let sig_alg = AlgorithmIdentifier {
            oid: sig_oid,
            parameters: None,
        };

        let name = Name::from_str(subject).expect("证书 DN 解析失败");

        let now = SystemTime::now();
        let not_before = now - Duration::from_secs(365 * 24 * 3600);
        let not_after = now + Duration::from_secs(365 * 24 * 3600);
        let validity: Validity = Validity::new(
            Time::try_from(not_before).expect("时间转换失败"),
            Time::try_from(not_after).expect("时间转换失败"),
        );

        let serial = SerialNumber::from(serial_number);

        // 4. 手工构造 TBS 证书 DER（TbsCertificate 字段为 pub(crate)，无法直接构造）
        let version_der = encode_version_v3();
        let serial_der = serial.to_der().expect("序列号编码失败");
        let sig_alg_der = sig_alg.to_der().expect("签名算法编码失败");
        let issuer_der = name.to_der().expect("颁发者编码失败");
        let validity_der = validity.to_der().expect("有效期编码失败");
        let subject_der = name.to_der().expect("使用者编码失败");
        let spki_der = spki.to_der().expect("公钥编码失败");

        // 4.1 构造 X.509 V3 Extensions（对应 Java: PKCGenerate GenCert 的 addExtensions）
        let extensions = build_x509_extensions();
        let exts_field = encode_extensions_tbs_field(&extensions);

        let tbs_content = [
            version_der.as_slice(),
            serial_der.as_slice(),
            sig_alg_der.as_slice(),
            issuer_der.as_slice(),
            validity_der.as_slice(),
            subject_der.as_slice(),
            spki_der.as_slice(),
            exts_field.as_slice(),
        ]
        .concat();
        let tbs_cert = der_sequence(&tbs_content);

        // 5. 使用 SM2withSM3 签名 TBS 证书
        use sm2::dsa::signature::Signer;
        let signature = signing_key.try_sign(&tbs_cert).expect("SM2 签名失败");
        let sig_bytes = signature.to_vec();

        // 6. 构造完整证书 DER: SEQUENCE { TBS, signatureAlgorithm, signature }
        let sig_bit_string = der_bit_string(&sig_bytes);
        let cert_content = [
            tbs_cert.as_slice(),
            sig_alg_der.as_slice(),
            sig_bit_string.as_slice(),
        ]
        .concat();
        der_sequence(&cert_content)
    }

    /// 生成默认序列号（基于当前时间戳毫秒数）。
    ///
    /// 使用 u128 -> u64 截断是安全的：毫秒时间戳在可预见的未来不会超过 u64 范围。
    #[allow(clippy::cast_possible_truncation)]
    fn default_serial() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// 编码 X.509 V3 版本字段: [0] EXPLICIT INTEGER(2)
///
/// DER 编码: A0 03 02 01 02
fn encode_version_v3() -> Vec<u8> {
    vec![0xA0, 0x03, 0x02, 0x01, 0x02]
}

/// 构造 DER SEQUENCE: tag(0x30) + length + content
fn der_sequence(content: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(2 + content.len());
    result.push(0x30); // SEQUENCE tag
    encode_der_length(&mut result, content.len());
    result.extend_from_slice(content);
    result
}

/// 构造 DER BIT STRING: tag(0x03) + length + unused_bits(0) + content
fn der_bit_string(content: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(3 + content.len());
    result.push(0x03); // BIT STRING tag
    encode_der_length(&mut result, content.len() + 1); // +1 for unused bits byte
    result.push(0x00); // unused bits = 0
    result.extend_from_slice(content);
    result
}

/// 编码 DER 长度字段。
///
/// DER 长度编码规则：
/// - 0..127: 单字节
/// - 128..256: 0x81 + 1 字节
/// - 256..65536: 0x82 + 2 字节
/// - 65536..16777216: 0x83 + 3 字节
#[allow(clippy::cast_possible_truncation)]
fn encode_der_length(buf: &mut Vec<u8>, len: usize) {
    if len < 0x80 {
        // SAFETY: len < 128，u8 不会截断
        buf.push(len as u8);
    } else if len < 0x100 {
        buf.push(0x81);
        // SAFETY: len < 256，u8 不会截断
        buf.push(len as u8);
    } else if len < 0x1_0000 {
        buf.push(0x82);
        // SAFETY: len < 65536，高 8 位不会截断
        buf.push((len >> 8) as u8);
        // SAFETY: 低 8 位不会截断
        buf.push(len as u8);
    } else if len < 0x0100_0000 {
        buf.push(0x83);
        // SAFETY: len < 16777216，高 8 位不会截断
        buf.push((len >> 16) as u8);
        buf.push((len >> 8) as u8);
        buf.push(len as u8);
    }
}

/// 构造 DER 上下文特定构造标签: [tag_num] constructed content
///
/// 用于 TBS 证书的 extensions 字段 `[3] EXPLICIT Extensions`。
/// context-specific tag 编码: 0xA0 | tag_num（constructed 位已置位）。
fn der_context_specific_constructed(tag_num: u8, content: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(2 + content.len());
    result.push(0xA0 | tag_num);
    encode_der_length(&mut result, content.len());
    result.extend_from_slice(content);
    result
}

/// 构造 TBS 证书的 X.509 V3 扩展列表。
///
/// 对应 Java: org.ofdrw.gm.cert.PKCGenerate GenCert 中的 addExtensions 调用。
///
/// 包含四个扩展，均设为非关键扩展（critical=false），与 Java 源码一致：
/// 1. BasicConstraints（cA=false）
/// 2. KeyUsage（digitalSignature | nonRepudiation | keyCertSign）
/// 3. ExtendedKeyUsage（clientAuth）
/// 4. NetscapeCertType（sslClient）
fn build_x509_extensions() -> Vec<Extension> {
    // 1. BasicConstraints (cA=false, 非关键扩展)
    // 对应 Java: .addExtension(Extension.basicConstraints, false, new BasicConstraints(false))
    // cA=false 为 DEFAULT 值，DER 编码中省略，结果为 SEQUENCE {}（空序列）
    let bc = BasicConstraints {
        ca: false,
        path_len_constraint: None,
    };
    let bc_oid =
        ObjectIdentifier::new(OID_BASIC_CONSTRAINTS).expect("BasicConstraints OID 解析失败");
    let bc_ext = Extension {
        extn_id: bc_oid,
        critical: false,
        extn_value: OctetString::new(bc.to_der().expect("BasicConstraints DER 编码失败"))
            .expect("BasicConstraints OctetString 构造失败"),
    };

    // 2. KeyUsage (digitalSignature | nonRepudiation | keyCertSign, 非关键扩展)
    // 对应 Java: .addExtension(Extension.keyUsage, false,
    //     new X509KeyUsage(X509KeyUsage.digitalSignature | nonRepudiation | keyCertSign))
    let ku =
        KeyUsage(KeyUsages::DigitalSignature | KeyUsages::NonRepudiation | KeyUsages::KeyCertSign);
    let ku_oid = ObjectIdentifier::new(OID_KEY_USAGE).expect("KeyUsage OID 解析失败");
    let ku_ext = Extension {
        extn_id: ku_oid,
        critical: false,
        extn_value: OctetString::new(ku.to_der().expect("KeyUsage DER 编码失败"))
            .expect("KeyUsage OctetString 构造失败"),
    };

    // 3. ExtendedKeyUsage (clientAuth, 非关键扩展)
    // 对应 Java: .addExtension(Extension.extendedKeyUsage, false,
    //     new ExtendedKeyUsage(KeyPurposeId.id_kp_clientAuth))
    let client_auth_oid =
        ObjectIdentifier::new(OID_KP_CLIENT_AUTH).expect("clientAuth OID 解析失败");
    let eku = ExtendedKeyUsage(vec![client_auth_oid]);
    let eku_oid =
        ObjectIdentifier::new(OID_EXTENDED_KEY_USAGE).expect("ExtendedKeyUsage OID 解析失败");
    let eku_ext = Extension {
        extn_id: eku_oid,
        critical: false,
        extn_value: OctetString::new(eku.to_der().expect("ExtendedKeyUsage DER 编码失败"))
            .expect("ExtendedKeyUsage OctetString 构造失败"),
    };

    // 4. NetscapeCertType (sslClient, 非关键扩展)
    // 对应 Java: .addExtension(MiscObjectIdentifiers.netscapeCertType, false,
    //     new NetscapeCertType(NetscapeCertType.sslClient))
    // BouncyCastle sslClient = 128 (0x80)，BIT STRING DER 编码: 03 02 00 80
    let netscape_oid =
        ObjectIdentifier::new(OID_NETSCAPE_CERT_TYPE).expect("NetscapeCertType OID 解析失败");
    let netscape_bit_string_der: Vec<u8> = vec![0x03, 0x02, 0x00, 0x80];
    let netscape_ext = Extension {
        extn_id: netscape_oid,
        critical: false,
        extn_value: OctetString::new(netscape_bit_string_der)
            .expect("NetscapeCertType OctetString 构造失败"),
    };

    vec![bc_ext, ku_ext, eku_ext, netscape_ext]
}

/// 将扩展列表编码为 TBS 证书中的 `[3] EXPLICIT Extensions` 字段。
///
/// 编码步骤：
/// 1. 逐个将 Extension 编码为 DER（SEQUENCE { extnID, critical?, extnValue }）
/// 2. 拼接为 SEQUENCE OF Extension 的内容
/// 3. 用 `der_sequence` 包装为完整 SEQUENCE
/// 4. 用 `der_context_specific_constructed` 添加 [3] EXPLICIT 标签
fn encode_extensions_tbs_field(extensions: &[Extension]) -> Vec<u8> {
    // 步骤 1+2: 编码每个 Extension 并拼接
    let mut exts_content = Vec::new();
    for ext in extensions {
        let ext_der = ext.to_der().expect("Extension DER 编码失败");
        exts_content.extend_from_slice(&ext_der);
    }

    // 步骤 3: 包装为 SEQUENCE OF Extension
    let exts_sequence = der_sequence(&exts_content);

    // 步骤 4: 添加 [3] EXPLICIT 标签
    der_context_specific_constructed(3, &exts_sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cert_tools::CertTools;

    #[test]
    fn test_generate_self_signed_not_empty() {
        let cert = PkcGenerate::generate_self_signed("CN=Test");
        assert!(!cert.is_empty());
    }

    #[test]
    fn test_generate_self_signed_parseable() {
        let cert_der = PkcGenerate::generate_self_signed("CN=Test Certificate");
        // 生成的证书应该能被解析
        let pk = CertTools::read_public_key(&cert_der);
        assert!(pk.is_some(), "应能从生成的证书中提取公钥");
        let pk_bytes = pk.unwrap();
        // SM2 未压缩公钥应该是 66 字节（unused_bits=0 + 0x04 + X + Y）
        assert!(
            pk_bytes.len() >= 65,
            "公钥长度应 >= 65，实际 {}",
            pk_bytes.len()
        );
    }

    #[test]
    fn test_generate_self_signed_validity() {
        let cert_der = PkcGenerate::generate_self_signed("CN=Test");
        // 当前时间应该在有效期内
        assert!(
            CertTools::check_validity(&cert_der),
            "生成的证书应该在有效期内"
        );
    }

    #[test]
    fn test_generate_self_signed_issuer_matches_subject() {
        let cert_der = PkcGenerate::generate_self_signed("CN=Test Cert,O=TestOrg");
        let issuer = CertTools::get_issuer(&cert_der);
        let subject = CertTools::get_subject(&cert_der);
        // 自签名证书的 issuer 和 subject 应该相同
        assert_eq!(issuer, subject, "自签名证书的 issuer 和 subject 应相同");
        assert!(
            issuer.contains("CN=Test Cert"),
            "issuer 应包含 CN=Test Cert"
        );
    }

    #[test]
    fn test_generate_self_signed_serial_number() {
        let cert_der = PkcGenerate::generate_self_signed_with_serial("CN=Test", 42);
        let serial = CertTools::get_serial_number(&cert_der);
        assert_eq!(serial, "2A", "序列号 42 应编码为 0x2A");
    }

    #[test]
    fn test_generate_self_signed_dn_format() {
        let cert_der = PkcGenerate::generate_self_signed(
            "CN=Test Certificate,O=OFD R&W,ST=Zhejiang,L=Hangzhou,C=CN",
        );
        let subject = CertTools::get_subject(&cert_der);
        assert!(subject.contains("CN=Test Certificate"), "subject 应包含 CN");
        assert!(subject.contains("O=OFD R&W"), "subject 应包含 O");
    }

    /// 验证生成的证书包含正确的 X.509 V3 扩展。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.PKCGenerate GenCert 的 addExtensions 验证。
    /// 使用 x509_cert::Certificate::from_der 解析证书，断言四个扩展均存在且值正确。
    #[test]
    fn test_generate_self_signed_has_extensions() {
        use x509_cert::Certificate;
        use x509_cert::der::Decode;

        let cert_der = PkcGenerate::generate_self_signed("CN=Test Extensions,O=TestOrg");
        let cert = Certificate::from_der(&cert_der).expect("证书 DER 解析失败");
        let tbs = cert.tbs_certificate();

        // 验证扩展字段存在
        let extensions = tbs.extensions().expect("证书应包含 extensions 字段");
        assert_eq!(extensions.len(), 4, "应包含 4 个扩展");

        // 验证 BasicConstraints: cA=false, 非关键扩展
        let (critical, bc) = tbs
            .get_extension::<BasicConstraints>()
            .expect("BasicConstraints 解析失败")
            .expect("应包含 BasicConstraints 扩展");
        assert!(!critical, "BasicConstraints 应为非关键扩展（与 Java 一致）");
        assert!(!bc.ca, "cA 应为 false");
        assert!(
            bc.path_len_constraint.is_none(),
            "pathLenConstraint 应为 None"
        );

        // 验证 KeyUsage: 包含 digitalSignature、nonRepudiation、keyCertSign
        let (critical, ku) = tbs
            .get_extension::<KeyUsage>()
            .expect("KeyUsage 解析失败")
            .expect("应包含 KeyUsage 扩展");
        assert!(!critical, "KeyUsage 应为非关键扩展（与 Java 一致）");
        assert!(ku.digital_signature(), "应包含 digitalSignature");
        assert!(ku.non_repudiation(), "应包含 nonRepudiation");
        assert!(ku.key_cert_sign(), "应包含 keyCertSign");

        // 验证 ExtendedKeyUsage 存在
        let (critical, _eku) = tbs
            .get_extension::<ExtendedKeyUsage>()
            .expect("ExtendedKeyUsage 解析失败")
            .expect("应包含 ExtendedKeyUsage 扩展");
        assert!(!critical, "ExtendedKeyUsage 应为非关键扩展（与 Java 一致）");

        // 验证 NetscapeCertType 存在（通过 OID 查找，x509-cert 无内置类型）
        let netscape_oid =
            ObjectIdentifier::new(OID_NETSCAPE_CERT_TYPE).expect("NetscapeCertType OID");
        let netscape_ext = extensions
            .iter()
            .find(|e| e.extn_id == netscape_oid)
            .expect("应包含 NetscapeCertType 扩展");
        assert!(
            !netscape_ext.critical,
            "NetscapeCertType 应为非关键扩展（与 Java 一致）"
        );
        // NetscapeCertType sslClient = 0x80，BIT STRING DER: 03 02 00 80
        assert_eq!(
            netscape_ext.extn_value.as_bytes(),
            &[0x03, 0x02, 0x00, 0x80],
            "NetscapeCertType 值应为 sslClient"
        );
    }
}
