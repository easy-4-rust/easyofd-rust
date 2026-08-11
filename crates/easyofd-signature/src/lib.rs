//! # easyofd-signature
//!
//! OFD electronic seal and digital signature operations per GB/T 38540.
//! Supports SM2WithSM3 (GM/T 0009) signature algorithm.

mod algorithm;
mod cert;
mod crl;
pub mod electronic_seal;
pub mod errors;
mod internal_helpers;
mod multi;
mod ofd_signature_builder;
mod read_signature;
mod seal;
pub mod sign_containers;
pub mod sign_dir;
pub mod sign_id_parser;
pub mod sign_id_provider;
mod signed_ofd;
pub mod signs_dir;
pub mod stamppos;
#[cfg(test)]
#[path = "test_lib.rs"]
mod tests;
mod timestamp;
pub mod timestamp_hook;
mod verification_result;
pub mod verify_containers;
mod verify_signature;
mod xml;

// Re-export `pub(crate)` helpers used by sibling modules (e.g. `multi.rs`).
pub(crate) use internal_helpers::{compute_sm3, hex, xml_escape};

// Re-export all public types and functions for `easyofd_signature::*` access.
pub use algorithm::SignatureAlgorithm;
pub use cert::{CertificateInfo, parse_x509_der, parse_x509_pem, verify_chain};
pub use crl::{CrlInfo, check_revoked, ocsp_check, ocsp_check_with_endpoint, parse_crl_der};
pub use electronic_seal::ElectronicSeal;
pub use errors::{
    DocNotSignException, FileIntegrityException, InvalidSignedValueException, OfdVerifyException,
    SignatureException, SignatureTerminateException,
};
pub use ofd_signature_builder::{
    DigitalSignContainer, OfdSignatureBuilder, SignMode, SignatureContainer, SignatureMethod,
};
pub use read_signature::read_signature;
pub use seal::{
    SealInfo, StampAppearance, StampSide, decode_seal_esl, encode_seal_esl, riding_stamp_appearance,
};
pub use sign_containers::{
    ExtendSignatureContainer, Gbt35275DsContainer, Gbt35275Pkcs9DsContainer, ProtectFileFilter,
    SesV1Container, SesV4Container, SesV5Container, SigType, SignCleaner, ToDigestFileInfo,
};
pub use sign_dir::SignDir;
pub use sign_id_parser::{NumberFormatAtomicSignId, SignIdParser, StandFormatAtomicSignId};
pub use sign_id_provider::{NumberSignIdProvider, SignIdProvider, StandardSignIdProvider};
pub use signed_ofd::SignedOfd;
pub use signs_dir::SignsDir;
pub use stamppos::{CuttingRatio, CuttingRideStampPos, NormalStampPos, RidingStampPos, Side};
pub use timestamp::{
    TimeStamp, create_timestamp, decode_der as decode_timestamp_der,
    encode_der as encode_timestamp_der,
};
pub use timestamp_hook::{ClosureTimeStampHook, TimeStampHook};
pub use verification_result::VerificationResult;
pub use verify_containers::{
    DigitalValidateContainer, Gbt35275ValidateContainer, OfdValidator, SesV1ValidateContainer,
    SesV4ValidateContainer, SesV5ValidateContainer, SignedDataValidateContainer,
};
pub use verify_signature::{SignatureVerificationResult, verify_signature, verify_signature_multi};

// ── Java 名称别名（对齐 ofdrw 命名） ─────────────────────────────────────────
//
// 以下别名用于 Java → Rust 迁移场景，使 Java 代码中的 `OFDVerifyException`、
// `GBT35275DSContainer` 等名称在 Rust 中可直接使用。
// trait 无法做 type alias，`SignIdProvider` 已通过 `pub use` 导出。

/// Java 名称别名：对应 `org.ofdrw.sign.verify.exceptions.OFDVerifyException`。
///
/// 已有 Rust 类型：[`OfdVerifyException`]。
pub type OFDVerifyException = OfdVerifyException;

/// Java 名称别名：对应 `org.ofdrw.sign.OFDSigner`。
///
/// 已有 Rust 类型：[`OfdSignatureBuilder`]。
pub type OFDSigner = OfdSignatureBuilder;

/// Java 名称别名：对应 `org.ofdrw.sign.signContainer.GBT35275DSContainer`。
///
/// 已有 Rust 类型：[`Gbt35275DsContainer`]。
pub type GBT35275DSContainer = Gbt35275DsContainer;

/// Java 名称别名：对应 `org.ofdrw.sign.signContainer.GBT35275PKCS9DSContainer`。
///
/// 已有 Rust 类型：[`Gbt35275Pkcs9DsContainer`]。
pub type GBT35275PKCS9DSContainer = Gbt35275Pkcs9DsContainer;

/// Java 名称别名：对应 `org.ofdrw.sign.signContainer.SESV1Container`。
///
/// 已有 Rust 类型：[`SesV1Container`]。
pub type SESV1Container = SesV1Container;

/// Java 名称别名：对应 `org.ofdrw.sign.signContainer.SESV4Container`。
///
/// 已有 Rust 类型：[`SesV4Container`]。
pub type SESV4Container = SesV4Container;

/// Java 名称别名：对应 `org.ofdrw.sign.signContainer.SESV5Container`。
///
/// 已有 Rust 类型：[`SesV5Container`]。
pub type SESV5Container = SesV5Container;

/// Java 名称别名：对应 `org.ofdrw.sign.NumberFormatAtomicSignID`。
///
/// 已有 Rust 类型：[`sign_id_parser::NumberFormatAtomicSignId`]。
pub type NumberFormatAtomicSignID = sign_id_parser::NumberFormatAtomicSignId;

/// Java 名称别名：对应 `org.ofdrw.sign.StandFormatAtomicSignID`。
///
/// 已有 Rust 类型：[`sign_id_parser::StandFormatAtomicSignId`]。
pub type StandFormatAtomicSignID = sign_id_parser::StandFormatAtomicSignId;

/// Java 名称别名：对应 `org.ofdrw.sign.verify.container.GBT35275ValidateContainer`。
///
/// 已有 Rust 类型：[`verify_containers::Gbt35275ValidateContainer`]。
pub type GBT35275ValidateContainer = Gbt35275ValidateContainer;

/// Java 名称别名：对应 `org.ofdrw.sign.verify.OFDValidator`。
///
/// 已有 Rust 类型：[`OfdValidator`]。
pub type OFDValidator = OfdValidator;

/// Java 名称别名：对应 `org.ofdrw.sign.verify.container.SESV1ValidateContainer`。
///
/// 已有 Rust 类型：[`SesV1ValidateContainer`]。
pub type SESV1ValidateContainer = SesV1ValidateContainer;

/// Java 名称别名：对应 `org.ofdrw.sign.verify.container.SESV4ValidateContainer`。
///
/// 已有 Rust 类型：[`SesV4ValidateContainer`]。
pub type SESV4ValidateContainer = SesV4ValidateContainer;

/// Java 名称别名：对应 `org.ofdrw.sign.verify.container.SESV5ValidateContainer`。
///
/// 已有 Rust 类型：[`SesV5ValidateContainer`]。
pub type SESV5ValidateContainer = SesV5ValidateContainer;

/// 对应 Java: SignIDProvider（Rust trait 别名）。
pub use SignIdProvider as SignIDProvider;
