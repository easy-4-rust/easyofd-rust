//! 容器文件过滤器。
//!
//! 对应 Java: org.ofdrw.crypto.ContainerFileFilter

/// 容器文件过滤器，决定 OFD 容器中的文件是否应被处理。
///
/// 对应 Java: `org.ofdrw.crypto.ContainerFileFilter`
///
/// 在加密/解密过程中，用于过滤哪些文件需要被加密或解密。
/// 例如，可以排除 `OFD.xml`（始终明文）或特定的资源文件。
pub trait ContainerFileFilter: std::fmt::Debug {
    /// 判断指定路径的文件是否应被处理。
    ///
    /// 返回 `true` 表示应被处理（加密/解密），`false` 表示跳过。
    fn should_process(&self, path: &str) -> bool;
}

/// 默认容器文件过滤器。
///
/// 跳过目录条目和加密描述文件（`EncryptInfo.xml`）。
#[derive(Debug, Clone, Copy)]
pub struct DefaultContainerFileFilter;

impl ContainerFileFilter for DefaultContainerFileFilter {
    fn should_process(&self, path: &str) -> bool {
        // 跳过目录条目
        if path.ends_with('/') {
            return false;
        }
        // 跳过加密描述文件
        if path == "EncryptInfo.xml" {
            return false;
        }
        true
    }
}

/// 白名单容器文件过滤器。
///
/// 只处理指定扩展名的文件。
#[derive(Debug, Clone)]
pub struct ExtensionFilter {
    /// 需要处理的文件扩展名列表（不含点号）。
    extensions: Vec<String>,
}

impl ExtensionFilter {
    /// 创建新的扩展名过滤器。
    #[must_use]
    pub fn new(extensions: Vec<String>) -> Self {
        Self {
            extensions: extensions.into_iter().map(|e| e.to_lowercase()).collect(),
        }
    }
}

impl ContainerFileFilter for ExtensionFilter {
    fn should_process(&self, path: &str) -> bool {
        if path.ends_with('/') {
            return false;
        }
        let ext = path.rsplit('.').next().unwrap_or("");
        self.extensions.contains(&ext.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter_skip_dir() {
        let filter = DefaultContainerFileFilter;
        assert!(!filter.should_process("Doc_0/Res/"));
    }

    #[test]
    fn test_default_filter_skip_encrypt_info() {
        let filter = DefaultContainerFileFilter;
        assert!(!filter.should_process("EncryptInfo.xml"));
    }

    #[test]
    fn test_default_filter_process_normal() {
        let filter = DefaultContainerFileFilter;
        assert!(filter.should_process("OFD.xml"));
        assert!(filter.should_process("Doc_0/Document.xml"));
        assert!(filter.should_process("Doc_0/Res/image.png"));
    }

    #[test]
    fn test_extension_filter() {
        let filter = ExtensionFilter::new(vec!["xml".into(), "png".into()]);
        assert!(filter.should_process("OFD.xml"));
        assert!(filter.should_process("image.PNG"));
        assert!(!filter.should_process("Doc_0/Res/"));
        assert!(!filter.should_process("document.pdf"));
    }

    #[test]
    fn test_extension_filter_case_insensitive() {
        let filter = ExtensionFilter::new(vec!["XML".into()]);
        assert!(filter.should_process("file.xml"));
        assert!(filter.should_process("file.XML"));
        assert!(filter.should_process("file.Xml"));
    }
}
