use crate::seal::SealInfo;

/// OFD 电子签章。
#[derive(Debug, Clone)]
pub struct ElectronicSeal {
    /// 签章图片数据（PNG 格式）。
    pub image_data: Vec<u8>,
    /// 签章名称。
    pub name: String,
    /// 签章在页面上的位置 (x, y)（mm）。
    pub position: (f64, f64),
    /// 签章所在页码（从 0 开始）。
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
