//! 签章子包（GB/T 33190 第 18 章）。
//!
//! 提供 OFD 数字签名和安全签章相关的数据类型：
//! - [`CheckMethod`] — 摘要算法枚举
//! - [`StampAnnot`] — 签章注释
//! - [`StampAnnotEntity`] — 签章注释实体
//! - [`Signatures`] — 签名列表根节点
//! - [`Signature`] — 签名注册信息
//! - [`SignedInfo`] — 签名信息
//! - [`References`] — 签名范围
//! - [`Reference`] — 文件摘要节点
//! - [`Seal`] — 电子印章信息

mod check_method;
mod reference;
mod references;
mod seal;
mod signature;
#[allow(clippy::module_inception)]
mod signatures;
mod signed_info;
mod stamp_annot;
mod stamp_annot_entity;

pub use check_method::CheckMethod;
pub use reference::Reference;
pub use references::References;
pub use seal::Seal;
pub use signature::{SigType, Signature};
pub use signatures::Signatures;
pub use signed_info::{Provider, SignedInfo};
pub use stamp_annot::StampAnnot;
pub use stamp_annot_entity::{SealImageType, StampAnnotEntity};
