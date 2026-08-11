//! OFD 容器路径。
//!
//! 对应 Java: org.ofdrw.crypto.ContainerPath
//!
//! 表示 OFD 加密容器中的文件路径，支持路径拼接和规范化。

/// OFD 容器路径，表示加密 OFD 文件系统中的路径。
///
/// 对应 Java: `org.ofdrw.crypto.ContainerPath`
///
/// 提供路径拼接、规范化和比较功能。路径使用 `/` 作为分隔符，
/// 以 `/` 开头的路径为绝对路径。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContainerPath {
    /// 规范化后的路径字符串。
    path: String,
}

impl ContainerPath {
    /// 创建新的容器路径。
    ///
    /// 对应 Java: `ContainerPath(String path)`
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            path: Self::normalize(&path),
        }
    }

    /// 获取路径字符串。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// 获取路径的文件名部分（最后一个 `/` 之后的内容）。
    ///
    /// 对应 Java: `ContainerPath.getFileName()`
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    /// 获取路径的父目录部分。
    ///
    /// 对应 Java: `ContainerPath.getParent()`
    #[must_use]
    pub fn parent(&self) -> Option<ContainerPath> {
        if self.path == "/" {
            return None;
        }
        self.path.rfind('/').map(|idx| {
            let parent = if idx == 0 { "/" } else { &self.path[..idx] };
            ContainerPath::new(parent)
        })
    }

    /// 拼接子路径。
    ///
    /// 对应 Java: `ContainerPath.cat(ST_Loc)`
    #[must_use]
    pub fn join(&self, child: &str) -> ContainerPath {
        if child.starts_with('/') {
            // 绝对路径直接返回
            ContainerPath::new(child)
        } else if self.path.ends_with('/') {
            ContainerPath::new(format!("{}{}", self.path, child))
        } else {
            ContainerPath::new(format!("{}/{}", self.path, child))
        }
    }

    /// 判断是否为绝对路径（以 `/` 开头）。
    #[must_use]
    pub fn is_absolute(&self) -> bool {
        self.path.starts_with('/')
    }

    /// 判断是否为根路径。
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.path == "/"
    }

    /// 规范化路径：移除多余的 `/`、处理 `.` 和 `..`。
    fn normalize(path: &str) -> String {
        let path = path.trim();
        if path.is_empty() {
            return "/".to_string();
        }

        let is_absolute = path.starts_with('/');
        let mut segments: Vec<&str> = Vec::new();

        for part in path.split('/') {
            let part = part.trim();
            if part.is_empty() || part == "." {
                continue;
            }
            if part == ".." {
                segments.pop();
            } else {
                segments.push(part);
            }
        }

        if segments.is_empty() {
            "/".to_string()
        } else if is_absolute {
            format!("/{}", segments.join("/"))
        } else {
            segments.join("/")
        }
    }
}

impl std::fmt::Display for ContainerPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path)
    }
}

impl From<String> for ContainerPath {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for ContainerPath {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for ContainerPath {
    fn as_ref(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_path_new() {
        let p = ContainerPath::new("/Doc_0/Res/image.png");
        assert_eq!(p.as_str(), "/Doc_0/Res/image.png");
        assert!(p.is_absolute());
        assert!(!p.is_root());
    }

    #[test]
    fn test_container_path_normalize() {
        assert_eq!(ContainerPath::new("/Doc_0//Res/").as_str(), "/Doc_0/Res");
        assert_eq!(ContainerPath::new("/Doc_0/./Res").as_str(), "/Doc_0/Res");
        assert_eq!(
            ContainerPath::new("/Doc_0/Pages/../Res").as_str(),
            "/Doc_0/Res"
        );
        assert_eq!(ContainerPath::new("/").as_str(), "/");
        assert_eq!(ContainerPath::new("").as_str(), "/");
    }

    #[test]
    fn test_container_path_file_name() {
        let p = ContainerPath::new("/Doc_0/Res/image.png");
        assert_eq!(p.file_name(), "image.png");
        assert_eq!(ContainerPath::new("/").file_name(), "");
    }

    #[test]
    fn test_container_path_parent() {
        let p = ContainerPath::new("/Doc_0/Res/image.png");
        let parent = p.parent().unwrap();
        assert_eq!(parent.as_str(), "/Doc_0/Res");

        let root = ContainerPath::new("/");
        assert!(root.parent().is_none());
    }

    #[test]
    fn test_container_path_join() {
        let base = ContainerPath::new("/Doc_0");
        let joined = base.join("Res/image.png");
        assert_eq!(joined.as_str(), "/Doc_0/Res/image.png");

        let abs = base.join("/Doc_1/Res");
        assert_eq!(abs.as_str(), "/Doc_1/Res");
    }

    #[test]
    fn test_container_path_display() {
        let p = ContainerPath::new("/Doc_0/Res");
        assert_eq!(format!("{p}"), "/Doc_0/Res");
    }

    #[test]
    fn test_container_path_from_string() {
        let p: ContainerPath = "/test/path".to_string().into();
        assert_eq!(p.as_str(), "/test/path");
    }

    #[test]
    fn test_container_path_from_str() {
        let p: ContainerPath = "/test/path".into();
        assert_eq!(p.as_str(), "/test/path");
    }

    #[test]
    fn test_container_path_as_ref() {
        let p = ContainerPath::new("/test");
        let s: &str = p.as_ref();
        assert_eq!(s, "/test");
    }

    #[test]
    fn test_container_path_eq() {
        let a = ContainerPath::new("/Doc_0/Res");
        let b = ContainerPath::new("/Doc_0/Res");
        assert_eq!(a, b);
    }

    #[test]
    fn test_container_path_relative() {
        let p = ContainerPath::new("Doc_0/Res");
        assert!(!p.is_absolute());
        assert_eq!(p.as_str(), "Doc_0/Res");
    }
}
