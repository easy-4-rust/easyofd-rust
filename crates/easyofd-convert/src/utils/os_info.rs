//! 操作系统平台检测。
//!
//! 对应 Java: org.ofdrw.converter.utils.OSinfo

/// 操作系统平台枚举。
///
/// 对应 Java `OSinfo.EPlatform`。用于在转换过程中根据操作系统
/// 选择合适的字体路径或文件分隔符等。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EPlatform {
    /// Linux。
    Linux,
    /// macOS（包括 OS X）。
    MacOS,
    /// Windows。
    Windows,
    /// FreeBSD。
    FreeBSD,
    /// Solaris / SunOS。
    Solaris,
    /// AIX。
    Aix,
    /// HP-UX。
    HpUx,
    /// 其他未知平台。
    Others,
}

impl EPlatform {
    /// 返回平台的可读描述。
    pub fn description(&self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::MacOS => "Mac OS",
            Self::Windows => "Windows",
            Self::FreeBSD => "FreeBSD",
            Self::Solaris => "Solaris",
            Self::Aix => "AIX",
            Self::HpUx => "HP-UX",
            Self::Others => "Others",
        }
    }

    /// 是否为类 Unix 系统。
    pub fn is_unix(&self) -> bool {
        matches!(
            self,
            Self::Linux | Self::MacOS | Self::FreeBSD | Self::Solaris | Self::Aix | Self::HpUx
        )
    }
}

impl std::fmt::Display for EPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

/// 检测当前操作系统平台。
///
/// 对应 Java `OSinfo.getOSname()`。
///
/// 使用编译时 `cfg` 属性判断，不依赖运行时环境变量。
pub fn current_platform() -> EPlatform {
    if cfg!(target_os = "linux") {
        EPlatform::Linux
    } else if cfg!(target_os = "macos") {
        EPlatform::MacOS
    } else if cfg!(target_os = "windows") {
        EPlatform::Windows
    } else if cfg!(target_os = "freebsd") {
        EPlatform::FreeBSD
    } else if cfg!(target_os = "solaris") {
        EPlatform::Solaris
    } else if cfg!(target_os = "aix") {
        EPlatform::Aix
    } else {
        EPlatform::Others
    }
}

/// 是否为 Linux 系统。
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// 是否为 macOS 系统。
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// 是否为 Windows 系统。
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// 获取系统临时目录路径。
///
/// 对应 Java `System.getProperty("java.io.tmpdir")`。
pub fn temp_dir_path() -> &'static str {
    // 编译时确定，避免运行时分配
    if cfg!(target_os = "windows") {
        "C:\\Windows\\Temp"
    } else {
        "/tmp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_platform_returns_value() {
        // 只验证不 panic，返回值与编译目标一致
        let platform = current_platform();
        let _desc = platform.description();
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(EPlatform::Linux.to_string(), "Linux");
        assert_eq!(EPlatform::MacOS.to_string(), "Mac OS");
        assert_eq!(EPlatform::Windows.to_string(), "Windows");
        assert_eq!(EPlatform::Others.to_string(), "Others");
    }

    #[test]
    fn test_is_unix() {
        assert!(EPlatform::Linux.is_unix());
        assert!(EPlatform::MacOS.is_unix());
        assert!(EPlatform::FreeBSD.is_unix());
        assert!(!EPlatform::Windows.is_unix());
        assert!(!EPlatform::Others.is_unix());
    }

    #[test]
    fn test_current_platform_matches_cfg() {
        let platform = current_platform();
        if cfg!(target_os = "linux") {
            assert_eq!(platform, EPlatform::Linux);
        } else if cfg!(target_os = "macos") {
            assert_eq!(platform, EPlatform::MacOS);
        } else if cfg!(target_os = "windows") {
            assert_eq!(platform, EPlatform::Windows);
        }
    }

    #[test]
    fn test_helper_functions_consistent() {
        let platform = current_platform();
        assert_eq!(is_linux(), platform == EPlatform::Linux);
        assert_eq!(is_macos(), platform == EPlatform::MacOS);
        assert_eq!(is_windows(), platform == EPlatform::Windows);
    }

    #[test]
    fn test_temp_dir_path_not_empty() {
        assert!(!temp_dir_path().is_empty());
    }

    #[test]
    fn test_platform_clone_eq() {
        let p1 = EPlatform::Linux;
        let p2 = p1;
        assert_eq!(p1, p2);
    }
}
