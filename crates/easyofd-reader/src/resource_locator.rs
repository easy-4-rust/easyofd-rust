//! 资源定位器，用于在 OFD 容器中定位和管理资源路径。
//!
//! 对应 Java: org.ofdrw.reader.ResourceLocator
//!
//! Java 版 `ResourceLocator` 基于解压后的文件系统目录导航。
//! Rust 版适配为 ZIP 归档内的虚拟路径导航，不依赖文件系统。

use crate::error_path_exception::ErrorPathException;

/// 资源定位器，维护 OFD 容器内的当前工作目录路径。
///
/// 对应 Java: `org.ofdrw.reader.ResourceLocator`
///
/// 提供 `cd`（切换目录）、`pwd`（打印当前目录）、`save`/`restore`
/// （保存/恢复目录栈）等操作，用于在 OFD ZIP 归档内构建绝对路径。
#[derive(Debug, Clone)]
pub struct ResourceLocator {
    /// 当前工作目录路径段。
    work_dir: Vec<String>,
    /// 保存的路径栈（每次 save 入栈，restore 出栈）。
    saved_stack: Vec<Vec<String>>,
}

impl ResourceLocator {
    /// 创建新的资源定位器，默认位于根目录 "/"。
    #[must_use]
    pub fn new() -> Self {
        Self {
            work_dir: vec!["/".to_string()],
            saved_stack: Vec::new(),
        }
    }

    /// 保存当前工作路径到栈中。
    ///
    /// 对应 Java: `ResourceLocator.save()`
    pub fn save(&mut self) {
        self.saved_stack.push(self.work_dir.clone());
    }

    /// 还原上一次保存的工作路径。
    ///
    /// 如果没有保存过路径，则不做任何操作。
    ///
    /// 对应 Java: `ResourceLocator.restore()`
    pub fn restore(&mut self) {
        if let Some(saved) = self.saved_stack.pop() {
            self.work_dir = saved;
        }
    }

    /// 切换到指定路径。
    ///
    /// 对应 Java: `ResourceLocator.cd(String)`
    ///
    /// # 错误
    ///
    /// 路径无效时返回 [`ErrorPathException`]。
    pub fn cd(&mut self, path: &str) -> Result<(), ErrorPathException> {
        if path.is_empty() {
            return Ok(());
        }
        let path = path.trim();
        if path == "/" {
            self.work_dir.clear();
            self.work_dir.push("/".to_string());
            return Ok(());
        }
        // 解析为绝对路径
        let abs_path = self.to_absolute_path(path);
        // 更新工作目录
        self.work_dir.clear();
        self.work_dir.push("/".to_string());
        for segment in abs_path.split('/') {
            let segment = segment.trim();
            if segment.is_empty() || segment == "." {
                continue;
            }
            self.work_dir.push(segment.to_string());
        }
        Ok(())
    }

    /// 重置工作路径到根目录。
    ///
    /// 对应 Java: `ResourceLocator.restWd()`
    pub fn reset(&mut self) {
        self.work_dir.clear();
        self.work_dir.push("/".to_string());
    }

    /// 打印当前工作目录路径。
    ///
    /// 对应 Java: `ResourceLocator.pwd()`
    #[must_use]
    pub fn pwd(&self) -> String {
        Self::pwd_of(&self.work_dir)
    }

    /// 将路径转换为绝对路径。
    ///
    /// 对应 Java: `ResourceLocator.toAbsolutePath(String)`
    #[must_use]
    pub fn to_absolute_path(&self, path: &str) -> String {
        if path.is_empty() {
            return self.pwd();
        }
        let path = path.trim();
        let mut segments: Vec<String> = if path.starts_with('/') {
            vec!["/".to_string()]
        } else {
            self.work_dir.clone()
        };

        for item in path.split('/') {
            let item = item.trim();
            if item == "." || item.is_empty() {
                continue;
            } else if item == ".." {
                segments.pop();
                if segments.is_empty() {
                    segments.push("/".to_string());
                }
            } else {
                segments.push(item.to_string());
            }
        }
        Self::pwd_of(&segments)
    }

    /// 获取以当前路径为基础的容器内绝对路径。
    ///
    /// 对应 Java: `ResourceLocator.getAbsTo(ST_Loc)`
    #[must_use]
    pub fn get_abs_to(&self, path: &str) -> String {
        if path.is_empty() {
            return self.pwd();
        }
        if path.starts_with('/') {
            return path.to_string();
        }
        // 查找最后一个 '/' 分隔文件名和目录部分
        if let Some(idx) = path.rfind('/') {
            let dir_part = &path[..idx + 1];
            let file_part = &path[idx + 1..];
            let mut wd = self.work_dir.clone();
            let abs_dir = Self::to_absolute_path_of(&wd, dir_part);
            if abs_dir.ends_with('/') {
                format!("{abs_dir}{file_part}")
            } else {
                format!("{abs_dir}/{file_part}")
            }
        } else {
            let pwd = self.pwd();
            if pwd.ends_with('/') {
                format!("{pwd}{path}")
            } else {
                format!("{pwd}/{path}")
            }
        }
    }

    /// 内部辅助：计算路径段列表的 pwd。
    fn pwd_of(segments: &[String]) -> String {
        if segments.len() <= 1 {
            return "/".to_string();
        }
        let mut result = String::new();
        for (i, item) in segments.iter().enumerate() {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            result.push_str(item);
            if item != "/" && i != segments.len() - 1 {
                result.push('/');
            }
        }
        result
    }

    /// 内部辅助：从给定 segments 计算绝对路径。
    fn to_absolute_path_of(segments: &[String], path: &str) -> String {
        if path.is_empty() {
            return Self::pwd_of(segments);
        }
        let path = path.trim();
        let mut work: Vec<String> = if path.starts_with('/') {
            vec!["/".to_string()]
        } else {
            segments.to_vec()
        };

        for item in path.split('/') {
            let item = item.trim();
            if item == "." || item.is_empty() {
                continue;
            } else if item == ".." {
                work.pop();
                if work.is_empty() {
                    work.push("/".to_string());
                }
            } else {
                work.push(item.to_string());
            }
        }
        Self::pwd_of(&work)
    }
}

impl Default for ResourceLocator {
    fn default() -> Self {
        Self::new()
    }
}

/// OFD 容器路径模式匹配辅助。
///
/// 对应 Java: `ResourceLocator` 中的静态 Pattern 字段。
pub mod patterns {
    /// 匹配 Doc_N 目录。
    pub fn is_doc_dir(segment: &str) -> bool {
        segment.starts_with("Doc_") && segment[4..].chars().all(|c| c.is_ascii_digit())
    }

    /// 匹配 Page_N 目录。
    pub fn is_page_dir(segment: &str) -> bool {
        segment.starts_with("Page_") && segment[5..].chars().all(|c| c.is_ascii_digit())
    }

    /// 匹配 Sign_N 目录。
    pub fn is_sign_dir(segment: &str) -> bool {
        segment.starts_with("Sign_") && segment[5..].chars().all(|c| c.is_ascii_digit())
    }

    /// 匹配 Res 目录。
    pub fn is_res_dir(segment: &str) -> bool {
        segment == "Res"
    }

    /// 匹配 Pages 目录。
    pub fn is_pages_dir(segment: &str) -> bool {
        segment == "Pages"
    }

    /// 匹配 Signs 目录。
    pub fn is_signs_dir(segment: &str) -> bool {
        segment == "Signs"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default_root() {
        let rl = ResourceLocator::new();
        assert_eq!(rl.pwd(), "/");
    }

    #[test]
    fn test_cd_root() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0").unwrap();
        assert_eq!(rl.pwd(), "/Doc_0");
        rl.cd("/").unwrap();
        assert_eq!(rl.pwd(), "/");
    }

    #[test]
    fn test_cd_relative() {
        let mut rl = ResourceLocator::new();
        rl.cd("Doc_0").unwrap();
        assert_eq!(rl.pwd(), "/Doc_0");
        rl.cd("Pages").unwrap();
        assert_eq!(rl.pwd(), "/Doc_0/Pages");
    }

    #[test]
    fn test_cd_parent() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0/Pages/Page_0").unwrap();
        rl.cd("..").unwrap();
        assert_eq!(rl.pwd(), "/Doc_0/Pages");
    }

    #[test]
    fn test_save_restore() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0").unwrap();
        rl.save();
        rl.cd("Pages").unwrap();
        assert_eq!(rl.pwd(), "/Doc_0/Pages");
        rl.restore();
        assert_eq!(rl.pwd(), "/Doc_0");
    }

    #[test]
    fn test_save_restore_nested() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0").unwrap();
        rl.save();
        rl.cd("Pages").unwrap();
        rl.save();
        rl.cd("Page_0").unwrap();
        assert_eq!(rl.pwd(), "/Doc_0/Pages/Page_0");
        rl.restore();
        assert_eq!(rl.pwd(), "/Doc_0/Pages");
        rl.restore();
        assert_eq!(rl.pwd(), "/Doc_0");
    }

    #[test]
    fn test_restore_empty_stack() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0").unwrap();
        // restore without save should not change anything
        rl.restore();
        assert_eq!(rl.pwd(), "/Doc_0");
    }

    #[test]
    fn test_to_absolute_path_absolute() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0/Pages").unwrap();
        assert_eq!(rl.to_absolute_path("/Doc_1/Res"), "/Doc_1/Res");
    }

    #[test]
    fn test_to_absolute_path_relative() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0").unwrap();
        assert_eq!(rl.to_absolute_path("Res/image.png"), "/Doc_0/Res/image.png");
    }

    #[test]
    fn test_to_absolute_path_parent() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0/Pages/Page_0").unwrap();
        assert_eq!(rl.to_absolute_path("../Res"), "/Doc_0/Pages/Res");
    }

    #[test]
    fn test_get_abs_to_absolute() {
        let rl = ResourceLocator::new();
        assert_eq!(rl.get_abs_to("/Doc_0/Res"), "/Doc_0/Res");
    }

    #[test]
    fn test_get_abs_to_relative() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0").unwrap();
        assert_eq!(rl.get_abs_to("Res/image.png"), "/Doc_0/Res/image.png");
    }

    #[test]
    fn test_get_abs_to_filename_only() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0/Pages/Page_0").unwrap();
        assert_eq!(
            rl.get_abs_to("Content.xml"),
            "/Doc_0/Pages/Page_0/Content.xml"
        );
    }

    #[test]
    fn test_reset() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0/Pages").unwrap();
        rl.reset();
        assert_eq!(rl.pwd(), "/");
    }

    #[test]
    fn test_cd_empty_string() {
        let mut rl = ResourceLocator::new();
        rl.cd("/Doc_0").unwrap();
        rl.cd("").unwrap();
        assert_eq!(rl.pwd(), "/Doc_0");
    }

    #[test]
    fn test_patterns() {
        assert!(patterns::is_doc_dir("Doc_0"));
        assert!(patterns::is_doc_dir("Doc_12"));
        assert!(!patterns::is_doc_dir("Doc_"));
        assert!(!patterns::is_doc_dir("Pages"));

        assert!(patterns::is_page_dir("Page_0"));
        assert!(patterns::is_page_dir("Page_99"));
        assert!(!patterns::is_page_dir("Page_"));

        assert!(patterns::is_sign_dir("Sign_0"));
        assert!(patterns::is_res_dir("Res"));
        assert!(patterns::is_pages_dir("Pages"));
        assert!(patterns::is_signs_dir("Signs"));
    }
}
