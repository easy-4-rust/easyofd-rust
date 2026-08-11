//! 完整性保护相关类型。
//!
//! 对应 Java: org.ofdrw.crypto.integrity

mod gm_protect_signer;
mod gm_protect_verifier;
mod ofd_integrity;
mod ofd_integrity_verifier;
mod protect_signer;
mod protect_verifier;

pub use gm_protect_signer::GmProtectSigner;
pub use gm_protect_verifier::GmProtectVerifier;
pub use ofd_integrity::OfdIntegrity;
pub use ofd_integrity_verifier::OfdIntegrityVerifier;
pub use protect_signer::{ProtectSigner, SimpleSigner};
pub use protect_verifier::{ProtectVerifier, SimpleVerifier};
