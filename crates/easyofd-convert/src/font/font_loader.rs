//! 字体加载器。
//!
//! 对应 Java: org.ofdrw.converter.FontLoader
//!
//! Java 版 `FontLoader` 负责从系统字体目录或指定路径加载字体，
//! 并维护字体名称到字体文件的映射。Rust 版提供简化的字体查找
//! 和加载功能。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 字体加载器。
///
/// 对应 Java: `org.ofdrw.converter.FontLoader`
///
/// 管理字体文件的查找路径和名称到路径的映射。
/// 支持从系统字体目录和用户指定目录加载字体。
#[derive(Debug, Clone)]
pub struct FontLoader {
    /// 字体搜索目录列表。
    search_dirs: Vec<PathBuf>,
    /// 字体名称到文件路径的缓存映射。
    font_map: HashMap<String, PathBuf>,
}

impl FontLoader {
    /// 创建空的字体加载器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            search_dirs: Vec::new(),
            font_map: HashMap::new(),
        }
    }

    /// 创建包含系统默认字体目录的加载器。
    ///
    /// 对应 Java: `FontLoader` 的默认构造行为。
    #[must_use]
    pub fn with_system_dirs() -> Self {
        let mut loader = Self::new();
        loader.add_system_font_dirs();
        loader
    }

    /// 添加字体搜索目录。
    ///
    /// 对应 Java: `FontLoader.addFontDir(Path dir)`。
    pub fn add_dir(&mut self, dir: impl Into<PathBuf>) {
        self.search_dirs.push(dir.into());
    }

    /// 添加多个字体搜索目录。
    pub fn add_dirs(&mut self, dirs: impl IntoIterator<Item = impl Into<PathBuf>>) {
        for dir in dirs {
            self.search_dirs.push(dir.into());
        }
    }

    /// 注册字体名称到文件路径的映射。
    ///
    /// 对应 Java: `FontLoader.addFont(String name, Path path)`。
    pub fn register_font(&mut self, name: impl Into<String>, path: impl Into<PathBuf>) {
        self.font_map.insert(name.into(), path.into());
    }

    /// 按字体名称查找字体文件路径。
    ///
    /// 对应 Java: `FontLoader.getFontPath(String fontName)`。
    ///
    /// 先查注册映射，再遍历搜索目录查找匹配文件。
    #[must_use]
    pub fn find_font(&self, name: &str) -> Option<PathBuf> {
        // 1. 查注册映射
        if let Some(path) = self.font_map.get(name)
            && path.exists()
        {
            return Some(path.clone());
        }

        // 2. 遍历搜索目录
        let lower_name = name.to_lowercase();
        for dir in &self.search_dirs {
            if !dir.is_dir() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy().to_lowercase();
                    let is_font = Path::new(&*file_name_str).extension().is_some_and(|ext| {
                        let e = ext.to_ascii_lowercase();
                        e == "ttf" || e == "otf" || e == "ttc"
                    });
                    if file_name_str.contains(&lower_name) && is_font {
                        return Some(entry.path());
                    }
                }
            }
        }

        None
    }

    /// 获取所有搜索目录。
    #[must_use]
    pub fn search_dirs(&self) -> &[PathBuf] {
        &self.search_dirs
    }

    /// 获取已注册的字体映射。
    #[must_use]
    pub fn font_map(&self) -> &HashMap<String, PathBuf> {
        &self.font_map
    }

    /// 清除所有注册的字体和搜索目录。
    pub fn clear(&mut self) {
        self.search_dirs.clear();
        self.font_map.clear();
    }

    /// 添加当前操作系统的默认字体目录。
    fn add_system_font_dirs(&mut self) {
        if cfg!(target_os = "macos") {
            self.search_dirs
                .push(PathBuf::from("/System/Library/Fonts"));
            self.search_dirs.push(PathBuf::from("/Library/Fonts"));
            if let Some(home) = std::env::var_os("HOME") {
                self.search_dirs
                    .push(Path::new(&home).join("Library/Fonts"));
            }
        } else if cfg!(target_os = "linux") {
            self.search_dirs.push(PathBuf::from("/usr/share/fonts"));
            self.search_dirs
                .push(PathBuf::from("/usr/local/share/fonts"));
        } else if cfg!(target_os = "windows")
            && let Some(win_dir) = std::env::var_os("WINDIR")
        {
            self.search_dirs.push(Path::new(&win_dir).join("Fonts"));
        }
    }
}

impl Default for FontLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let loader = FontLoader::new();
        assert!(loader.search_dirs().is_empty());
        assert!(loader.font_map().is_empty());
    }

    #[test]
    fn test_default() {
        let loader = FontLoader::default();
        assert!(loader.search_dirs().is_empty());
    }

    #[test]
    fn test_add_dir() {
        let mut loader = FontLoader::new();
        loader.add_dir("/usr/share/fonts");
        assert_eq!(loader.search_dirs().len(), 1);
    }

    #[test]
    fn test_add_dirs() {
        let mut loader = FontLoader::new();
        loader.add_dirs(["/usr/share/fonts", "/usr/local/share/fonts"]);
        assert_eq!(loader.search_dirs().len(), 2);
    }

    #[test]
    fn test_register_font() {
        let mut loader = FontLoader::new();
        loader.register_font("SimSun", "/usr/share/fonts/simsun.ttf");
        assert!(loader.font_map().contains_key("SimSun"));
    }

    #[test]
    fn test_find_font_registered() {
        // 使用一个实际存在的路径注册字体
        let temp_dir = std::env::temp_dir();
        let font_path = temp_dir.join("test_font.ttf");
        std::fs::write(&font_path, b"fake font data").unwrap();

        let mut loader = FontLoader::new();
        loader.register_font("TestFont", &font_path);

        let found = loader.find_font("TestFont");
        assert!(found.is_some());
        assert_eq!(found.as_ref().unwrap(), &font_path);

        let _ = std::fs::remove_file(&font_path);
    }

    #[test]
    fn test_find_font_not_found() {
        let loader = FontLoader::new();
        assert!(loader.find_font("NonExistentFont").is_none());
    }

    #[test]
    fn test_clear() {
        let mut loader = FontLoader::new();
        loader.add_dir("/usr/share/fonts");
        loader.register_font("Test", "/test.ttf");
        loader.clear();
        assert!(loader.search_dirs().is_empty());
        assert!(loader.font_map().is_empty());
    }
}
