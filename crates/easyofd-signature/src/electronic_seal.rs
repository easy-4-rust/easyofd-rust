use crate::seal::SealInfo;

/// OFD 电子签章。
#[derive(Debug, Clone)]
pub struct ElectronicSeal {
    pub image_data: Vec<u8>,
    pub name: String,
    pub position: (f64, f64),
    pub page: usize,
}

/// 将旧版 [`ElectronicSeal`] 转换为 [`SealInfo`]，保留向后兼容。
///
/// `position` 和 `page` 字段在当前签名流程中未使用，因此不映射到 `SealInfo`。
impl From<ElectronicSeal> for SealInfo {
    fn from(e: ElectronicSeal) -> Self {
        SealInfo {
            name: e.name,
            created_at: chrono::Utc::now(),
            valid_until: chrono::Utc::now() + chrono::Duration::days(365),
            cert_der: Vec::new(),
            image: e.image_data,
            version: 1,
        }
    }
}
