//! GB/T 35275 国密签名数据结构的对象标识符（OID）常量。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.OIDs

/// GB/T 35275 SM2 基础 OID。
pub const GMT35275_SM2: &str = "1.2.156.10197.6.1.4.2";
/// data 内容类型 OID。
pub const DATA: &str = "1.2.156.10197.6.1.4.2.1";
/// signedData 内容类型 OID。
pub const SIGNED_DATA: &str = "1.2.156.10197.6.1.4.2.2";
/// envelopedData 内容类型 OID。
pub const ENVELOPED_DATA: &str = "1.2.156.10197.6.1.4.2.3";
/// signedAndEnvelopedData 内容类型 OID。
pub const SIGNED_AND_ENVELOPED_DATA: &str = "1.2.156.10197.6.1.4.2.4";
/// encryptedData 内容类型 OID。
pub const ENCRYPTED_DATA: &str = "1.2.156.10197.6.1.4.2.5";
/// keyAgreementInfo 内容类型 OID。
pub const KEY_AGREEMENT_INFO: &str = "1.2.156.10197.6.1.4.2.6";
/// SM4 算法 OID。
pub const SM4: &str = "1.2.156.10197.1.100";
/// SM2 算法族 OID。
pub const SM2: &str = "1.2.156.10197.1.301";
/// SM2 签名算法 OID（SM2 与 SM3）。
pub const SM2_SIGN: &str = "1.2.156.10197.1.301.1";
/// SM2 密钥交换 OID。
pub const SM2_KEY_EXCHANGE: &str = "1.2.156.10197.1.301.2";
/// SM2 加密 OID。
pub const SM2_ENCRYPT: &str = "1.2.156.10197.1.301.3";
/// SM3 摘要算法 OID。
pub const SM3: &str = "1.2.156.10197.1.401";

/// 将点分 OID 字符串解析为弧段数组（如 `"1.2.840.113549.1.1.11"` → `[1, 2, 840, ...]`）。
#[must_use]
pub fn parse_oid(s: &str) -> Vec<u32> {
    s.split('.')
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

/// 将弧段数组编码为点分 OID 字符串。
#[must_use]
pub fn format_oid(arcs: &[u32]) -> String {
    arcs.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_format_oid() {
        let arcs = parse_oid(SM2_SIGN);
        assert_eq!(arcs, [1, 2, 156, 10_197, 1, 301, 1]);
        assert_eq!(format_oid(&arcs), SM2_SIGN);
    }

    #[test]
    fn test_signed_data_oid() {
        let arcs = parse_oid(SIGNED_DATA);
        assert_eq!(format_oid(&arcs), SIGNED_DATA);
    }
}
