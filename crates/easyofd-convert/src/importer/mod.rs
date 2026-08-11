//! 导入器模块。

#[allow(clippy::module_inception)]
pub mod importer;
pub mod pdf_importer;

pub use importer::Importer;
pub use pdf_importer::PdfImporter;
