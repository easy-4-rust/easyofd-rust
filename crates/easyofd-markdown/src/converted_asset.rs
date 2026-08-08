use std::path::PathBuf;

/// 转换过程中导出的资源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertedAsset {
    /// 资源来源页码。
    pub page: usize,
    /// 页面中的来源对象下标。
    pub object_index: usize,
    /// 实际写入路径。
    pub path: PathBuf,
}
