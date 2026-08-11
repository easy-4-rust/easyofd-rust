//! OFD 包容器类型。
//!
//! 对应 Java: org.ofdrw.pkg.container

pub mod annots_dir;
pub mod ofd_dir;
pub mod ofd_package_file_iterator;
pub mod page_dir;
pub mod pages_dir;
pub mod res_dir;
pub mod temps_dir;
pub mod virtual_container;

pub use annots_dir::AnnotsDir;
pub use ofd_dir::OfdPkgDir;
pub use ofd_package_file_iterator::OfdPackageFileIterator;
pub use page_dir::PageDir;
pub use pages_dir::PagesDir;
pub use res_dir::ResDir;
pub use temps_dir::TempsDir;
pub use virtual_container::VirtualContainer;
