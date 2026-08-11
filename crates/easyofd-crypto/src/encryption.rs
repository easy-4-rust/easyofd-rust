//! 加密描述结构，对应 Java 版 [`ofdrw-crypto`](https://github.com/ofdrw/ofdrw) 中的加密元数据。
//!
//! 参考 GB/T 33190 第 19 章加密规范。

// CT_EncryptInfo 保留 Java 原始命名（org.ofdrw.crypto.encryt.CT_EncryptInfo）。
#![allow(non_camel_case_types)]

/// 加密方案标识（GB/T 33190 表 28）。
///
/// 对应 Java: `org.ofdrw.crypto.encryt.ProtectionCaseID`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtectionCaseID {
    /// SM4-CBC 模式（推荐）。
    Sm4Cbc,
    /// SM4-ECB 模式。
    Sm4Ecb,
}

impl ProtectionCaseID {
    /// 返回方案标识字符串，与 GB/T 33190 表 28 一致。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sm4Cbc => "SM4-CBC",
            Self::Sm4Ecb => "SM4-ECB",
        }
    }
}

impl std::fmt::Display for ProtectionCaseID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 加密信息，描述 OFD 文档使用的加密方案及参数。
///
/// 对应 Java: `org.ofdrw.crypto.encryt.CT_EncryptInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CT_EncryptInfo {
    /// 加密方案标识。
    pub protection_case_id: ProtectionCaseID,
    /// 加密参数（如 IV、盐值等，由具体方案决定）。
    pub encrypt_info: Vec<u8>,
}

impl CT_EncryptInfo {
    /// 创建新的加密信息实例。
    #[must_use]
    pub fn new(protection_case_id: ProtectionCaseID, encrypt_info: Vec<u8>) -> Self {
        Self {
            protection_case_id,
            encrypt_info,
        }
    }
}

/// 明密文映射表，记录 OFD 文档中所有被加密文件的路径映射。
///
/// 对应 Java: `org.ofdrw.crypto.encryt.EncryptEntries`
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncryptEntries {
    /// 明密文映射条目列表。
    pub entries: Vec<EncryptEntry>,
}

impl EncryptEntries {
    /// 创建空的映射表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一条映射。
    pub fn push(&mut self, entry: EncryptEntry) {
        self.entries.push(entry);
    }

    /// 返回映射条目数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 映射表是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 单条明密文映射，记录一个文件从原文路径到密文路径的对应关系。
///
/// 对应 Java: `org.ofdrw.crypto.encryt.EncryptEntry`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptEntry {
    /// 原始文件路径（OFD ZIP 内相对路径）。
    pub original_path: String,
    /// 加密后文件路径（OFD ZIP 内相对路径）。
    pub encrypted_path: String,
}

impl EncryptEntry {
    /// 创建新的映射条目。
    #[must_use]
    pub fn new(original_path: String, encrypted_path: String) -> Self {
        Self {
            original_path,
            encrypted_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_case_id_display() {
        assert_eq!(ProtectionCaseID::Sm4Cbc.to_string(), "SM4-CBC");
        assert_eq!(ProtectionCaseID::Sm4Ecb.to_string(), "SM4-ECB");
    }

    #[test]
    fn test_protection_case_id_as_str() {
        assert_eq!(ProtectionCaseID::Sm4Cbc.as_str(), "SM4-CBC");
        assert_eq!(ProtectionCaseID::Sm4Ecb.as_str(), "SM4-ECB");
    }

    #[test]
    fn test_ct_encrypt_info_new() {
        let info = CT_EncryptInfo::new(ProtectionCaseID::Sm4Cbc, vec![1, 2, 3]);
        assert_eq!(info.protection_case_id, ProtectionCaseID::Sm4Cbc);
        assert_eq!(info.encrypt_info, vec![1, 2, 3]);
    }

    #[test]
    fn test_encrypt_entries_push_and_len() {
        let mut entries = EncryptEntries::new();
        assert!(entries.is_empty());
        assert_eq!(entries.len(), 0);

        entries.push(EncryptEntry::new("a.xml".into(), "a.xml.enc".into()));
        assert_eq!(entries.len(), 1);
        assert!(!entries.is_empty());

        entries.push(EncryptEntry::new("b.png".into(), "b.png.enc".into()));
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_encrypt_entry_new() {
        let entry = EncryptEntry::new("Doc.xml".into(), "Doc.xml.enc".into());
        assert_eq!(entry.original_path, "Doc.xml");
        assert_eq!(entry.encrypted_path, "Doc.xml.enc");
    }

    #[test]
    fn test_protection_case_id_clone_eq() {
        let a = ProtectionCaseID::Sm4Cbc;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_ct_encrypt_info_clone_eq() {
        let a = CT_EncryptInfo::new(ProtectionCaseID::Sm4Cbc, vec![0xAA; 16]);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_encrypt_entries_default_is_empty() {
        let entries = EncryptEntries::default();
        assert!(entries.is_empty());
    }
}
