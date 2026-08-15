use easyofd_core::OfdResult;
use std::io::{Cursor, Read, Write};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::algorithm::SignatureAlgorithm;
use crate::electronic_seal::ElectronicSeal;
use crate::internal_helpers::{base64_encode, compute_sm3, hex, xml_escape};
use crate::seal::SealInfo;
use crate::signed_ofd::SignedOfd;
use crate::timestamp::TimeStamp;
use crate::{multi, seal, timestamp, xml};

// ── 签名容器体系 ────────────────────────────────────────────────────────

/// 签名方法（对应 ofdrw `SignatureMethod`）。
///
/// 用于 [`SignatureContainer`] 返回当前容器使用的签名算法标识。
pub type SignatureMethod = SignatureAlgorithm;

/// 签名容器接口（对应 ofdrw `ExtendSignatureContainer`）。
///
/// 不同的容器实现不同的 `SignedValue.dat` 格式：
/// - [`DigitalSignContainer`]: 原始 SM2 签名值 + base64
/// - `GBT35275DSContainer`: GB/T 35275 CMS SignedData（预留）
/// - `SESV1Container`: GM/T 0031 SES_Signature（预留）
/// - `SESV4Container`: GB/T 38540 SES_Signature（预留）
/// - `SESV5Container`: V5 版 SES_Signature（预留）
pub trait SignatureContainer: Send + Sync {
    /// 返回签名算法 OID（如 `"1.2.156.10197.1.501"` 对应 SM2WithSM3）。
    fn algorithm_oid(&self) -> &str;

    /// 返回签名方法枚举。
    fn signature_method(&self) -> SignatureMethod;

    /// 根据 SignedInfo 字节和私钥构建 SignedValue.dat 内容。
    fn build_signed_value(
        &self,
        signed_info_bytes: &[u8],
        sk: &sm2::SecretKey,
    ) -> OfdResult<Vec<u8>>;

    /// 验证签名值是否与 SignedInfo 字节和公钥匹配。
    fn verify(
        &self,
        signed_info_bytes: &[u8],
        signed_value: &[u8],
        pk: &sm2::dsa::VerifyingKey,
    ) -> bool;
}

// ── 签名模式 ─────────────────────────────────────────────────────────────

/// 签名模式（对应 ofdrw-sign `SignMode`）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SignMode {
    /// 整体保护：现有 Signature.xml 包含在摘要范围内，禁止后续追加。
    #[default]
    WholeProtected,
    /// 追加签名：跳过现有 Signature.xml，允许后续签章。
    ContinueSign,
}

// ── DigitalSignContainer 默认实现 ─────────────────────────────────────────

/// 数字签名容器（对应 ofdrw `DigitalSignContainer`）。
///
/// 使用裸 SM2 签名 + hex 编码作为 `SignedValue.dat` 格式，
/// 保持与现有签名流程的向后兼容。
///
/// 算法 OID: `1.2.156.10197.1.501`（SM2WithSM3）。
pub struct DigitalSignContainer {
    algorithm: SignatureAlgorithm,
}

impl DigitalSignContainer {
    /// 创建指定算法的数字签名容器。
    #[must_use]
    pub fn new(algorithm: SignatureAlgorithm) -> Self {
        Self { algorithm }
    }
}

impl Default for DigitalSignContainer {
    fn default() -> Self {
        Self {
            algorithm: SignatureAlgorithm::Sm2WithSm3,
        }
    }
}

impl SignatureContainer for DigitalSignContainer {
    fn algorithm_oid(&self) -> &str {
        match self.algorithm {
            SignatureAlgorithm::Sm2WithSm3 => "1.2.156.10197.1.501",
            SignatureAlgorithm::Sha256WithRsa => "1.2.840.113549.1.1.11",
        }
    }

    fn signature_method(&self) -> SignatureMethod {
        self.algorithm
    }

    fn build_signed_value(
        &self,
        signed_info_bytes: &[u8],
        sk: &sm2::SecretKey,
    ) -> OfdResult<Vec<u8>> {
        use sm2::dsa::signature::Signer;
        let signing_key = sm2::dsa::SigningKey::new("1234567812345678", sk)
            .map_err(|e| easyofd_core::OfdError::Conversion(format!("SM2 密钥派生失败: {e}")))?;
        let sig = signing_key.sign(signed_info_bytes);
        Ok(sig.to_bytes().to_vec())
    }

    fn verify(
        &self,
        signed_info_bytes: &[u8],
        signed_value: &[u8],
        pk: &sm2::dsa::VerifyingKey,
    ) -> bool {
        let Ok(signature) = sm2::dsa::Signature::from_slice(signed_value) else {
            return false;
        };
        use sm2::dsa::signature::Verifier;
        pk.verify(signed_info_bytes, &signature).is_ok()
    }
}

/// 签章构建器。
pub struct OfdSignatureBuilder {
    input_path: String,
    seals: Vec<SealInfo>,
    algorithm: SignatureAlgorithm,
    multi_mode: bool,
    signer_entries: Vec<multi::SignerEntry>,
    timestamp: Option<TimeStamp>,
    sign_mode: SignMode,
}

impl OfdSignatureBuilder {
    /// 创建签名构建器，指定待签名的 OFD 文件路径。
    #[must_use]
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input_path: input.into(),
            seals: Vec::new(),
            algorithm: SignatureAlgorithm::Sm2WithSm3,
            multi_mode: false,
            signer_entries: Vec::new(),
            timestamp: None,
            sign_mode: SignMode::default(),
        }
    }
    /// 添加签章信息。
    #[must_use]
    pub fn seal(mut self, seal: impl Into<SealInfo>) -> Self {
        self.seals.push(seal.into());
        self
    }
    /// 设置签名算法。
    #[must_use]
    pub fn algorithm(mut self, alg: SignatureAlgorithm) -> Self {
        self.algorithm = alg;
        self
    }

    /// Enable multi-signer mode.
    #[must_use]
    pub fn with_multiple_seals(mut self, enabled: bool) -> Self {
        self.multi_mode = enabled;
        self
    }

    /// Add an independent signer (with their own SM2 secret key and seals).
    #[must_use]
    pub fn add_signature(mut self, secret_key: sm2::SecretKey, seals: Vec<ElectronicSeal>) -> Self {
        self.signer_entries
            .push(multi::SignerEntry { secret_key, seals });
        self
    }

    /// Attach a timestamp token to the signature.
    ///
    /// When set, `<ofd:TimeStamp>BASE64(DER)</ofd:TimeStamp>` is inserted
    /// into the Signature.xml after the `<ofd:SignedInfoRef>` element.
    #[must_use]
    pub fn with_timestamp(mut self, ts: TimeStamp) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// 设置签名模式。
    ///
    /// - [`SignMode::WholeProtected`]（默认）：整体保护，包含所有文件摘要。
    /// - [`SignMode::ContinueSign`]：追加签名，排除已有的 Signature.xml。
    #[must_use]
    pub fn sign_mode(mut self, mode: SignMode) -> Self {
        self.sign_mode = mode;
        self
    }

    /// Produce multiple independent signatures in one pass.
    ///
    /// Each signer receives `Signature_<n>.xml`, `SignedInfo_<n>.xml`, and
    /// `SignedValue_<n>.dat`.  The OFD.xml is augmented with a
    /// `<ofd:Signatures>` element listing all `SignatureRef` entries.
    pub fn sign_multiple(self) -> OfdResult<SignedOfd> {
        if self.signer_entries.is_empty() {
            return Err(easyofd_core::OfdError::Conversion(
                "multi-signer mode requires at least one signer".into(),
            ));
        }
        let input_bytes = std::fs::read(&self.input_path).map_err(easyofd_core::OfdError::Io)?;
        let cursor = Cursor::new(&input_bytes[..]);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        easyofd_package::validate_archive(&mut archive, easyofd_package::PackageLimits::default())?;
        let mut entry_data: Vec<(String, Vec<u8>)> = Vec::with_capacity(archive.len());
        for i in 0..archive.len() {
            let mut e = archive
                .by_index(i)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            let name = e.name().to_string();
            let mut buf = Vec::new();
            e.read_to_end(&mut buf)
                .map_err(easyofd_core::OfdError::Io)?;
            entry_data.push((name, buf));
        }
        let (data, _) =
            multi::sign_multiple_impl(&entry_data, &self.signer_entries, self.algorithm)?;
        Ok(SignedOfd {
            data,
            digest: String::new(),
            signature_value: String::new(),
        })
    }

    /// 执行签名，返回签名后的 OFD。
    ///
    /// # 错误
    ///
    /// OFD 文件读取、签章数据或签名操作失败时返回错误。
    #[allow(clippy::too_many_lines)]
    pub fn sign(self) -> OfdResult<SignedOfd> {
        let input_bytes = std::fs::read(&self.input_path).map_err(easyofd_core::OfdError::Io)?;

        // 1. 加载原始 OFD 包，先按规范对每个被保护 entry 计算 SM3 摘要。
        let cursor = Cursor::new(&input_bytes[..]);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        easyofd_package::validate_archive(&mut archive, easyofd_package::PackageLimits::default())?;

        let mut entry_data: Vec<(String, Vec<u8>)> = Vec::with_capacity(archive.len());
        for i in 0..archive.len() {
            let mut e = archive
                .by_index(i)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            let name = e.name().to_string();
            let mut buf = Vec::new();
            e.read_to_end(&mut buf)
                .map_err(easyofd_core::OfdError::Io)?;
            entry_data.push((name, buf));
        }

        // 2. 对每个 entry 计算 SM3 摘要，构造 SignedInfo。
        //    GB/T 38540: References/FileRef 列出每个受保护文件 + CheckValue。
        //    为简化解析（避免 `<ofd:RootFile FileRef>` 这种嵌套结构对读取器的
        //    字符串扫描造成歧义），所有 entry 一律作为 `<ofd:FileRef>`。
        //    Doc_0/Document.xml 的"根文件"语义由 References 顺序蕴含。
        //
        //    当 sign_mode == ContinueSign 时，排除已有的 Signature.xml，
        //    允许后续追加签名（对应 ofdrw SignMode.ContinueSign）。
        let protected_entries: Vec<(String, Vec<u8>)> = match self.sign_mode {
            SignMode::WholeProtected => entry_data.clone(),
            SignMode::ContinueSign => entry_data
                .iter()
                .filter(|(name, _)| !is_signature_file(name))
                .cloned()
                .collect(),
        };
        let provider = "easyofd-rust";
        let signature_method = match self.algorithm {
            SignatureAlgorithm::Sm2WithSm3 => "SM2WithSM3",
            SignatureAlgorithm::Sha256WithRsa => "SHA256WithRSA",
        };
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let mut file_refs = String::new();
        for (name, data) in &protected_entries {
            use std::fmt::Write as _;
            let _ = write!(
                file_refs,
                r#"<ofd:FileRef CheckMethod="SM3" CheckValue="{}">{}</ofd:FileRef>"#,
                hex(&compute_sm3(data)),
                xml_escape(name)
            );
        }

        // 3. 组装 SignedInfo 字节（这一段字节才是 SM2 签名的真实消息）。
        let signed_info_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Provider Version="1.0">{}</ofd:Provider>
  <ofd:SignatureMethod>{}</ofd:SignatureMethod>
  <ofd:SignatureDateTime>{}</ofd:SignatureDateTime>
  <ofd:SealCount>{}</ofd:SealCount>
  <ofd:References>
    {}
  </ofd:References>
</ofd:SignedInfo>"#,
            provider,
            signature_method,
            now,
            self.seals.len(),
            file_refs,
        );

        // 4. 用签名容器对 SignedInfo 字节做签名。
        //    默认使用 DigitalSignContainer（裸 SM2 签名），保持向后兼容。
        use sm2::elliptic_curve::Generate;
        let secret_key = sm2::SecretKey::generate();
        let container = DigitalSignContainer::new(self.algorithm);
        let signed_info_bytes = signed_info_xml.as_bytes();
        let signed_value = container.build_signed_value(signed_info_bytes, &secret_key)?;
        let sig_hex = hex(&signed_value);
        let signing_key = sm2::dsa::SigningKey::new("1234567812345678", &secret_key)
            .map_err(|e| easyofd_core::OfdError::Conversion(format!("SM2 密钥派生失败: {e}")))?;
        let pub_hex = hex(&signing_key.verifying_key().to_sec1_bytes());

        // 5. 构建输出 ZIP：先原样写入所有原始 entry，再追加 Seal/SignedValue.dat/SignedInfo/Signature.xml。
        let out = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(out);
        let opts =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name, data) in &entry_data {
            zip.start_file(name, opts)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(data).map_err(easyofd_core::OfdError::Io)?;
        }
        for (i, seal) in self.seals.iter().enumerate() {
            zip.start_file(format!("Doc_0/Res/Seal_{i}.png"), opts)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(&seal.image)
                .map_err(easyofd_core::OfdError::Io)?;
            // 写入 Seal_<n>.esl ASN.1 DER 容器（GB/T 38540 §5.4）
            let esl_bytes = seal::encode_seal_esl(seal)?;
            zip.start_file(format!("Doc_0/Seal_{i}.esl"), opts)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(&esl_bytes)
                .map_err(easyofd_core::OfdError::Io)?;
        }
        // SignedValue.dat：签名值二进制。GB/T 38540 要求与 Signature.xml 分离。
        zip.start_file("Doc_0/Signs/SignedValue.dat", opts)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(&signed_value)
            .map_err(easyofd_core::OfdError::Io)?;
        // SignedInfo.xml：摘要 + References。
        zip.start_file("Doc_0/Signs/SignedInfo.xml", opts)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(signed_info_bytes)
            .map_err(easyofd_core::OfdError::Io)?;
        // Signature.xml：顶级封装，包含 SignedInfo 引用 + SignedValue 引用 + 公钥 + 印章。
        let signature_xml = build_signature_xml(
            &signed_info_xml,
            &self.seals,
            self.algorithm,
            &pub_hex,
            &sig_hex,
            self.timestamp.as_ref(),
        )?;
        zip.start_file("Doc_0/Signs/Signature.xml", opts)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(signature_xml.as_bytes())
            .map_err(easyofd_core::OfdError::Io)?;

        let data = zip
            .finish()
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?
            .into_inner();
        Ok(SignedOfd {
            data,
            digest: hex(&compute_sm3(signed_info_bytes)),
            signature_value: sig_hex,
        })
    }
}

/// 构造顶层 Signature.xml：
/// 引用 SignedInfo 路径、SignedValue 路径、可选印章 ID/类型，承载公钥，
/// 可选时间戳。
fn build_signature_xml(
    signed_info_xml: &str,
    seals: &[SealInfo],
    algorithm: SignatureAlgorithm,
    pub_hex: &str,
    sig_hex: &str,
    timestamp: Option<&TimeStamp>,
) -> OfdResult<String> {
    // 提取 SignedInfo 的 Provider/Method/DateTime 字段，避免重复声明（GB/T 38540）。
    let si_top = xml::parse_signature_top(signed_info_xml)?;
    let method = si_top.method.unwrap_or_else(|| "SM2WithSM3".to_string());
    let provider = si_top
        .provider
        .unwrap_or_else(|| "easyofd-rust".to_string());
    let datetime = si_top.datetime.unwrap_or_default();
    let _ = algorithm; // method 已从 signed_info 读取，参数避免未用告警

    // 印章：每枚印章生成一个 SignedValue.dat/SignedInfo.xml 双文件，按
    // 当前 v0.6 简化设计只放一张总签章，所有 seal 都嵌为 Resource。
    let mut seal_list = String::new();
    for (i, _seal) in seals.iter().enumerate() {
        use std::fmt::Write as _;
        let _ = write!(
            seal_list,
            r#"<ofd:Seal ID="Seal_{i}" Type="Seal" Ref="Doc_0/Seal_{i}.esl">Doc_0/Res/Seal_{i}.png</ofd:Seal>"#
        );
    }

    // KeyInfo：当签章者配置了 cert_der 时，嵌入 X.509 证书（GB/T 38540 §4）。
    let key_info = seals
        .iter()
        .find(|s| !s.cert_der.is_empty())
        .map(|s| {
            format!(
                "  <ofd:KeyInfo>\n    <ofd:Certificate>{}</ofd:Certificate>\n  </ofd:KeyInfo>\n",
                base64_encode(&s.cert_der)
            )
        })
        .unwrap_or_default();

    let timestamp_xml = timestamp
        .map(|ts| match timestamp::encode_der(ts) {
            Ok(der_bytes) => format!(
                "  <ofd:TimeStamp>{}</ofd:TimeStamp>\n",
                base64_encode(&der_bytes)
            ),
            Err(_) => String::new(),
        })
        .unwrap_or_default();

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016" ID="Signature_0">
  <ofd:SignedInfoRef>Doc_0/Signs/SignedInfo.xml</ofd:SignedInfoRef>
{timestamp_xml}  <ofd:SignedValue>Doc_0/Signs/SignedValue.dat</ofd:SignedValue>
{key_info}  <ofd:Provider Version="1.0">{provider}</ofd:Provider>
  <ofd:SignatureMethod>{method}</ofd:SignatureMethod>
  <ofd:SignatureDateTime>{datetime}</ofd:SignatureDateTime>
  {seal_list}
  <ofd:PublicKey>{pub_hex}</ofd:PublicKey>
  <ofd:SignatureValue>{sig_hex}</ofd:SignatureValue>
</ofd:Signature>"#
    ))
}

/// 判断文件路径是否为签名文件（Signature.xml 或 Signature_*.xml）。
///
/// 用于 [`SignMode::ContinueSign`] 模式下排除已有签名，允许追加签章。
pub(crate) fn is_signature_file(name: &str) -> bool {
    name.ends_with("/Signature.xml")
        || (name.contains("/Signature_")
            && std::path::Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("xml")))
}
