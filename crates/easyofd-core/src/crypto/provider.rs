//! 加密组件提供者（Provider）。
//!
//! 对应 Java: org.ofdrw.core.crypto.encryt.Provider
//!
//! GMT0099 C.3 表 C.2，描述加密组件的相关信息。
//! 与签名组件 Provider（`signatures::Provider`）的区别在于：
//! 加密 Provider 使用 `Name` 属性，签名 Provider 使用 `ProviderName` 属性。

/// 加密组件提供者。
///
/// 对应 Java: `org.ofdrw.core.crypto.encryt.Provider`
///
/// 描述加密组件的相关信息（GMT0099 C.3 表 C.2）。
/// 与签名组件 [`signatures::Provider`](crate::signatures::Provider) 不同，
/// 此类型使用 `Name` 属性而非 `ProviderName`。
#[derive(Debug, Clone)]
pub struct CryptoProvider {
    /// 加密组件名称（必选）。
    name: String,
    /// 加密组件版本（可选）。
    version: Option<String>,
    /// 加密组件制造商（可选）。
    company: Option<String>,
    /// 接口协议版本（可选，OFD 2.0）。
    protocol_ver: Option<String>,
    /// 扩展信息（可选，OFD 2.0），Base64 编码前的原始字节。
    extend_data: Option<Vec<u8>>,
}

impl CryptoProvider {
    /// 创建加密组件提供者。
    ///
    /// # 参数
    ///
    /// - `name`：加密组件名称（必选）。
    ///
    /// # 对应 Java
    ///
    /// `org.ofdrw.core.crypto.encryt.Provider#setProviderName(String)`
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            company: None,
            protocol_ver: None,
            extend_data: None,
        }
    }

    /// 获取加密组件名称。
    ///
    /// 对应 Java: `getProviderName()` → 读取 `Name` 属性
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 设置加密组件版本。
    ///
    /// 对应 Java: `setVersion(String)`
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// 获取加密组件版本。
    #[must_use]
    pub fn version_ref(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// 设置加密组件制造商。
    ///
    /// 对应 Java: `setCompany(String)`
    #[must_use]
    pub fn company(mut self, company: impl Into<String>) -> Self {
        self.company = Some(company.into());
        self
    }

    /// 获取加密组件制造商。
    #[must_use]
    pub fn company_ref(&self) -> Option<&str> {
        self.company.as_deref()
    }

    /// 设置接口协议版本（OFD 2.0）。
    ///
    /// 对应 Java: `setProtocolVer(String)`
    #[must_use]
    pub fn protocol_ver(mut self, ver: impl Into<String>) -> Self {
        self.protocol_ver = Some(ver.into());
        self
    }

    /// 获取接口协议版本。
    #[must_use]
    pub fn protocol_ver_ref(&self) -> Option<&str> {
        self.protocol_ver.as_deref()
    }

    /// 设置扩展信息（OFD 2.0）。
    ///
    /// 原始字节在序列化时会被 Base64 编码。
    ///
    /// 对应 Java: `setExtendData(byte[])`
    #[must_use]
    pub fn extend_data(mut self, data: Vec<u8>) -> Self {
        self.extend_data = Some(data);
        self
    }

    /// 获取扩展信息原始字节。
    #[must_use]
    pub fn extend_data_ref(&self) -> Option<&[u8]> {
        self.extend_data.as_deref()
    }

    /// 序列化为 XML 字符串。
    ///
    /// 产出格式与 Java `Provider.toString()` 一致：
    /// `<ofd:Provider Name="..." Version="..." Company="..." ProtocolVer="...">`
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;

        let mut xml = format!(r#"<ofd:Provider Name="{}""#, self.name);
        if let Some(ref v) = self.version {
            let _ = write!(xml, r#" Version="{v}""#);
        }
        if let Some(ref c) = self.company {
            let _ = write!(xml, r#" Company="{c}""#);
        }
        if let Some(ref p) = self.protocol_ver {
            let _ = write!(xml, r#" ProtocolVer="{p}""#);
        }
        if let Some(ref data) = self.extend_data {
            let encoded = base64_encode(data);
            let _ = write!(
                xml,
                "><ofd:ExtendData>{encoded}</ofd:ExtendData></ofd:Provider>"
            );
        } else {
            xml.push_str(" />");
        }
        xml
    }
}

/// 简单的 Base64 编码实现（避免引入额外依赖）。
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = if chunk.len() > 1 {
            u32::from(chunk[1])
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            u32::from(chunk[2])
        } else {
            0
        };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_provider() {
        let p = CryptoProvider::new("TestCrypto");
        assert_eq!(p.name(), "TestCrypto");
        assert!(p.version_ref().is_none());
        assert!(p.company_ref().is_none());
        assert!(p.protocol_ver_ref().is_none());
        assert!(p.extend_data_ref().is_none());
    }

    #[test]
    fn builder_chain() {
        let p = CryptoProvider::new("SM4Provider")
            .version("1.0")
            .company("ACME")
            .protocol_ver("2.0");
        assert_eq!(p.name(), "SM4Provider");
        assert_eq!(p.version_ref(), Some("1.0"));
        assert_eq!(p.company_ref(), Some("ACME"));
        assert_eq!(p.protocol_ver_ref(), Some("2.0"));
    }

    #[test]
    fn xml_name_only() {
        let p = CryptoProvider::new("TestProvider");
        let xml = p.to_xml_string();
        assert!(xml.contains(r#"Name="TestProvider""#));
        assert!(xml.contains("<ofd:Provider"));
        assert!(xml.ends_with("/>"));
    }

    #[test]
    fn xml_all_attributes() {
        let p = CryptoProvider::new("SM4")
            .version("1.0")
            .company("TestCo")
            .protocol_ver("2.0");
        let xml = p.to_xml_string();
        assert!(xml.contains(r#"Name="SM4""#));
        assert!(xml.contains(r#"Version="1.0""#));
        assert!(xml.contains(r#"Company="TestCo""#));
        assert!(xml.contains(r#"ProtocolVer="2.0""#));
    }

    #[test]
    fn xml_with_extend_data() {
        let p = CryptoProvider::new("Test").extend_data(vec![0x01, 0x02, 0x03]);
        let xml = p.to_xml_string();
        assert!(xml.contains("<ofd:ExtendData>"));
        assert!(xml.contains("</ofd:Provider>"));
        // Base64 of [1,2,3] = "AQID"
        assert!(xml.contains("AQID"));
    }

    #[test]
    fn base64_encode_basic() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn clone_provider() {
        let p = CryptoProvider::new("Test").version("1.0");
        let cloned = p.clone();
        assert_eq!(cloned.name(), "Test");
        assert_eq!(cloned.version_ref(), Some("1.0"));
    }

    #[test]
    fn extend_data_ref() {
        let p = CryptoProvider::new("Test").extend_data(vec![10, 20, 30]);
        assert_eq!(p.extend_data_ref(), Some(&[10u8, 20, 30][..]));
    }
}
