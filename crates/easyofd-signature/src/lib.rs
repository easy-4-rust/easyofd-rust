//! # easyofd-signature
//!
//! OFD electronic seal and digital signature operations per GB/T 38540.
//! Supports SM2WithSM3 (GM/T 0009) signature algorithm.

mod algorithm;
mod cert;
mod crl;
mod electronic_seal;
mod internal_helpers;
mod multi;
mod ofd_signature_builder;
mod read_signature;
mod seal;
mod signed_ofd;
#[cfg(test)]
#[path = "test_lib.rs"]
mod tests;
mod timestamp;
mod verification_result;
mod verify_signature;
mod xml;

// Re-export `pub(crate)` helpers used by sibling modules (e.g. `multi.rs`).
pub(crate) use internal_helpers::{compute_sm3, hex, xml_escape};

// Re-export all public types and functions for `easyofd_signature::*` access.
pub use algorithm::SignatureAlgorithm;
pub use cert::{CertificateInfo, parse_x509_der, parse_x509_pem, verify_chain};
pub use crl::{CrlInfo, check_revoked, ocsp_check, ocsp_check_with_endpoint, parse_crl_der};
pub use electronic_seal::ElectronicSeal;
pub use ofd_signature_builder::{
    DigitalSignContainer, OfdSignatureBuilder, SignMode, SignatureContainer, SignatureMethod,
};
pub use read_signature::read_signature;
pub use seal::{
    SealInfo, StampAppearance, StampSide, decode_seal_esl, encode_seal_esl, riding_stamp_appearance,
};
pub use signed_ofd::SignedOfd;
pub use timestamp::{
    TimeStamp, create_timestamp, decode_der as decode_timestamp_der,
    encode_der as encode_timestamp_der,
};
pub use verification_result::VerificationResult;
pub use verify_signature::{SignatureVerificationResult, verify_signature, verify_signature_multi};
