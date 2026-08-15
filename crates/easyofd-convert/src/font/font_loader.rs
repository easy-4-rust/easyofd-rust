//! 字体加载器。
//!
//! 对应 Java: org.ofdrw.converter.FontLoader
//!
//! Java 版 `FontLoader` 负责从系统字体目录或指定路径加载字体，
//! 并维护字体名称到字体文件的映射。Rust 版提供简化的字体查找
//! 和加载功能，支持相似字体正则替换回退。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 简化模式匹配器。
///
/// 对应 Java: `java.util.regex.Pattern`
///
/// Java 版使用 `Pattern.compile(regex)` 编译正则表达式，
/// 但默认注册的模式均为 `.*X.*` 形式（子串匹配）。
///
/// 此处实现轻量级模式匹配，覆盖 Java 默认表的实际模式：
/// - `*X` → `ends_with("X")`
/// - `X*` → `starts_with("X")`
/// - `*X*` → `contains("X")`（Java `.*X.*` 的等价形式）
/// - `X` → 精确匹配
///
/// 若后续需要完整正则支持，可替换为 `regex` crate。
#[derive(Debug, Clone)]
enum SimilarPattern {
    /// 子串匹配：`.*X.*` 或 `*X*`
    Contains(String),
    /// 前缀匹配：`X*`
    StartsWith(String),
    /// 后缀匹配：`*X`
    EndsWith(String),
    /// 精确匹配
    Exact(String),
}

impl SimilarPattern {
    /// 从 Java 风格正则或 glob 模式创建匹配器。
    ///
    /// 支持两种格式：
    /// - Java 正则：`.*X.*` → Contains("X")
    /// - Glob 通配符：`*X*`、`X*`、`*X`、`X`
    fn from_pattern(pattern: &str) -> Self {
        // 尝试解析 Java 正则 `.*X.*`
        if let Some(inner) = parse_dot_star_pattern(pattern) {
            return Self::Contains(inner);
        }
        // Glob 通配符
        if let Some(inner) = pattern.strip_prefix('*').and_then(|s| s.strip_suffix('*')) {
            if !inner.is_empty() {
                return Self::Contains(inner.to_string());
            }
        }
        if let Some(inner) = pattern.strip_suffix('*') {
            if !inner.is_empty() {
                return Self::StartsWith(inner.to_string());
            }
        }
        if let Some(inner) = pattern.strip_prefix('*') {
            if !inner.is_empty() {
                return Self::EndsWith(inner.to_string());
            }
        }
        Self::Exact(pattern.to_string())
    }

    /// 测试输入字符串是否匹配。
    fn is_match(&self, input: &str) -> bool {
        match self {
            Self::Contains(sub) => input.contains(sub.as_str()),
            Self::StartsWith(prefix) => input.starts_with(prefix.as_str()),
            Self::EndsWith(suffix) => input.ends_with(suffix.as_str()),
            Self::Exact(exact) => input == exact.as_str(),
        }
    }
}

/// 尝试解析 Java 正则 `.*X.*` 形式，返回中间的字面量 X。
fn parse_dot_star_pattern(regex: &str) -> Option<String> {
    let rest = regex.strip_prefix(".*")?;
    let inner = rest.strip_suffix(".*")?;
    // 内部不应含正则元字符（默认表中均为字面量）
    if inner.contains(|c: char| {
        matches!(
            c,
            '\\' | '[' | ']' | '(' | ')' | '{' | '}' | '+' | '?' | '^' | '$' | '|'
        )
    }) {
        return None;
    }
    Some(inner.to_string())
}

/// 字体加载器。
///
/// 对应 Java: `org.ofdrw.converter.FontLoader`
///
/// 管理字体文件的查找路径和名称到路径的映射。
/// 支持从系统字体目录和用户指定目录加载字体。
/// 支持相似字体正则替换回退。
#[derive(Debug, Clone)]
pub struct FontLoader {
    /// 字体搜索目录列表。
    search_dirs: Vec<PathBuf>,
    /// 字体名称到文件路径的缓存映射。
    ///
    /// 对应 Java: `fontNamePathMapping`
    font_map: HashMap<String, PathBuf>,
    /// 字体别名映射（名称 → 别名）。
    ///
    /// 对应 Java: `fontNameAliasMapping`
    alias_map: HashMap<String, String>,
    /// 相似字体正则替换映射（模式 → 替换目标字体名）。
    ///
    /// 对应 Java: `similarFontReplaceRegexMapping: Map<Pattern, String>`
    similar_font_replace: Vec<(SimilarPattern, String)>,
    /// 是否启用相似字体替换。
    ///
    /// 对应 Java: `enableSimilarFontReplace`
    enable_similar_font_replace: bool,
}

impl FontLoader {
    /// 创建空的字体加载器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            search_dirs: Vec::new(),
            font_map: HashMap::new(),
            alias_map: HashMap::new(),
            similar_font_replace: Vec::new(),
            enable_similar_font_replace: true,
        }
    }

    /// 创建包含系统默认字体目录和默认替换规则的加载器。
    ///
    /// 对应 Java: `FontLoader.init()` 中的默认初始化逻辑。
    ///
    /// 注册默认别名映射和相似字体替换规则：
    /// - 别名：小标宋体→方正小标宋简体、KaiTi_GB2312→楷体、楷体→KaiTi、宋体→SimSun
    /// - 相似替换：`.*Kai.*`→楷体、`.*MinionPro.*`→SimSun、`.*SimSun.*`→SimSun、`.*Song.*`→宋体
    #[must_use]
    pub fn with_system_dirs() -> Self {
        let mut loader = Self::new();
        loader.add_system_font_dirs();
        loader.init_default_mappings();
        loader
    }

    /// 注册 Java 版 `init()` 中的默认别名和相似替换规则。
    ///
    /// 对应 Java: `FontLoader.init()` 中 `addAliasMapping` 和 `addSimilarFontReplaceRegexMapping` 调用。
    fn init_default_mappings(&mut self) {
        // ── 默认别名映射 ──
        // 对应 Java: addAliasMapping("小标宋体", "方正小标宋简体")
        self.add_alias_mapping("小标宋体", "方正小标宋简体");
        // 对应 Java: addAliasMapping("KaiTi_GB2312", "楷体")
        self.add_alias_mapping("KaiTi_GB2312", "楷体");
        // 对应 Java: addAliasMapping("楷体", "KaiTi")
        self.add_alias_mapping("楷体", "KaiTi");
        // 对应 Java: addAliasMapping("宋体", "SimSun")
        self.add_alias_mapping("宋体", "SimSun");

        // ── 默认相似字体替换规则 ──
        // 对应 Java: addSimilarFontReplaceRegexMapping(".*Kai.*", "楷体")
        self.add_similar_font_replace_regex_mapping(".*Kai.*", "楷体");
        // 对应 Java: addSimilarFontReplaceRegexMapping(".*MinionPro.*", "SimSun")
        self.add_similar_font_replace_regex_mapping(".*MinionPro.*", "SimSun");
        // 对应 Java: addSimilarFontReplaceRegexMapping(".*SimSun.*", "SimSun")
        self.add_similar_font_replace_regex_mapping(".*SimSun.*", "SimSun");
        // 对应 Java: addSimilarFontReplaceRegexMapping(".*Song.*", "宋体")
        self.add_similar_font_replace_regex_mapping(".*Song.*", "宋体");
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

    /// 添加字体别名映射。
    ///
    /// 对应 Java: `FontLoader.addAliasMapping(String fontName, String alias)`
    ///
    /// 当直接查找字体名失败时，会尝试通过别名映射查找。
    pub fn add_alias_mapping(&mut self, font_name: impl Into<String>, alias: impl Into<String>) {
        self.alias_map.insert(font_name.into(), alias.into());
    }

    /// 添加相似字体正则替换规则。
    ///
    /// 对应 Java: `FontLoader.addSimilarFontReplaceRegexMapping(String fontNameRegex, String fontName)`
    ///
    /// 当正常字体解析（直接查找 + 别名查找）全部失败时，
    /// 按注册顺序遍历此映射，找到第一个匹配的模式后使用其替换目标。
    ///
    /// # 模式格式
    ///
    /// - Java 正则 `.*X.*` 形式 → 子串匹配
    /// - Glob `*X*` / `X*` / `*X` → 通配符匹配
    /// - 其他 → 精确匹配
    pub fn add_similar_font_replace_regex_mapping(
        &mut self,
        font_name_regex: impl Into<String>,
        font_name: impl Into<String>,
    ) {
        let pattern_str = font_name_regex.into();
        let target = font_name.into();
        if pattern_str.is_empty() || target.is_empty() {
            return;
        }
        let pattern = SimilarPattern::from_pattern(&pattern_str);
        self.similar_font_replace.push((pattern, target));
    }

    /// 设置是否启用相似字体替换。
    ///
    /// 对应 Java: `FontLoader.setSimilarFontReplace(boolean enable)`
    pub fn set_similar_font_replace(&mut self, enable: bool) {
        self.enable_similar_font_replace = enable;
    }

    /// 是否启用了相似字体替换。
    #[must_use]
    pub fn is_similar_font_replace_enabled(&self) -> bool {
        self.enable_similar_font_replace
    }

    /// 按字体名称查找字体文件路径。
    ///
    /// 对应 Java: `FontLoader.getSystemFontPath(String familyName, String fontName)`
    ///
    /// 查找顺序：
    /// 1. 直接查 `font_map`
    /// 2. 通过 `alias_map` 查别名
    #[must_use]
    pub fn find_font(&self, name: &str) -> Option<PathBuf> {
        // 1. 查注册映射
        if let Some(path) = self.font_map.get(name)
            && path.exists()
        {
            return Some(path.clone());
        }

        // 2. 查别名映射
        if let Some(alias) = self.alias_map.get(name) {
            if let Some(path) = self.font_map.get(alias)
                && path.exists()
            {
                return Some(path.clone());
            }
        }

        // 3. 遍历搜索目录
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

    /// 带相似字体替换的字体查找。
    ///
    /// 对应 Java: `FontLoader.getReplaceSimilarFontPath(String familyName, String fontName)`
    ///
    /// 查找顺序（对齐 Java 回退链）：
    /// 1. 直接查找（`find_font`，含别名）
    /// 2. 若 `enable_similar_font_replace` 为 true，遍历相似替换规则匹配
    /// 3. 仍失败返回 `None`
    #[must_use]
    pub fn find_font_with_similar_replace(
        &self,
        family_name: Option<&str>,
        font_name: Option<&str>,
    ) -> Option<PathBuf> {
        // 1. 尝试直接通过字体名查找
        if let Some(name) = font_name {
            if let Some(path) = self.find_font(name) {
                return Some(path);
            }
        }
        // 尝试通过字族名查找
        if let Some(family) = family_name {
            if let Some(path) = self.find_font(family) {
                return Some(path);
            }
        }

        // 2. 相似字体替换
        if !self.enable_similar_font_replace {
            return None;
        }

        for (pattern, target) in &self.similar_font_replace {
            let matched = font_name.is_some_and(|n| pattern.is_match(n))
                || family_name.is_some_and(|f| pattern.is_match(f));
            if matched {
                // 用替换目标再查一次
                if let Some(path) = self.find_font(target) {
                    return Some(path);
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

    /// 获取别名映射。
    #[must_use]
    pub fn alias_map(&self) -> &HashMap<String, String> {
        &self.alias_map
    }

    /// 获取相似替换规则数量。
    #[must_use]
    pub fn similar_replace_count(&self) -> usize {
        self.similar_font_replace.len()
    }

    /// 清除所有注册的字体、搜索目录、别名和相似替换规则。
    pub fn clear(&mut self) {
        self.search_dirs.clear();
        self.font_map.clear();
        self.alias_map.clear();
        self.similar_font_replace.clear();
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

    // ── 基础功能（原有测试） ──

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
        loader.add_alias_mapping("A", "B");
        loader.add_similar_font_replace_regex_mapping(".*X.*", "Y");
        loader.clear();
        assert!(loader.search_dirs().is_empty());
        assert!(loader.font_map().is_empty());
        assert!(loader.alias_map().is_empty());
        assert_eq!(loader.similar_replace_count(), 0);
    }

    // ── 别名映射 ──

    #[test]
    fn test_alias_mapping() {
        let temp_dir = std::env::temp_dir();
        let font_path = temp_dir.join("simsun.ttf");
        std::fs::write(&font_path, b"fake font data").unwrap();

        let mut loader = FontLoader::new();
        loader.register_font("SimSun", &font_path);
        loader.add_alias_mapping("宋体", "SimSun");

        // 通过别名 "宋体" 应能找到 SimSun 的路径
        let found = loader.find_font("宋体");
        assert!(found.is_some());
        assert_eq!(found.as_ref().unwrap(), &font_path);

        let _ = std::fs::remove_file(&font_path);
    }

    // ── 相似字体替换 ──

    #[test]
    fn test_similar_font_replace_default_rules() {
        let temp_dir = std::env::temp_dir();
        let font_path = temp_dir.join("kaiti.ttf");
        std::fs::write(&font_path, b"fake kaiti data").unwrap();

        let mut loader = FontLoader::new();
        // 注册 "楷体" 字体
        loader.register_font("楷体", &font_path);
        // 添加默认相似替换规则
        loader.add_similar_font_replace_regex_mapping(".*Kai.*", "楷体");

        // "KaiTi" 未直接注册，应通过相似替换找到 "楷体"
        let found = loader.find_font_with_similar_replace(None, Some("KaiTi"));
        assert!(found.is_some());
        assert_eq!(found.as_ref().unwrap(), &font_path);

        let _ = std::fs::remove_file(&font_path);
    }

    #[test]
    fn test_similar_font_replace_by_family_name() {
        let temp_dir = std::env::temp_dir();
        let font_path = temp_dir.join("simsun.ttf");
        std::fs::write(&font_path, b"fake simsun data").unwrap();

        let mut loader = FontLoader::new();
        loader.register_font("SimSun", &font_path);
        loader.add_similar_font_replace_regex_mapping(".*SimSun.*", "SimSun");

        // 通过 family_name 匹配
        let found = loader.find_font_with_similar_replace(Some("SimSun-Regular"), None);
        assert!(found.is_some());

        let _ = std::fs::remove_file(&font_path);
    }

    #[test]
    fn test_similar_font_replace_disabled() {
        let temp_dir = std::env::temp_dir();
        let font_path = temp_dir.join("kaiti.ttf");
        std::fs::write(&font_path, b"fake kaiti data").unwrap();

        let mut loader = FontLoader::new();
        loader.register_font("楷体", &font_path);
        loader.add_similar_font_replace_regex_mapping(".*Kai.*", "楷体");

        // 关闭相似替换
        loader.set_similar_font_replace(false);
        assert!(!loader.is_similar_font_replace_enabled());

        // "KaiTi" 未注册且替换关闭，应返回 None
        let found = loader.find_font_with_similar_replace(None, Some("KaiTi"));
        assert!(found.is_none());

        let _ = std::fs::remove_file(&font_path);
    }

    #[test]
    fn test_explicit_mapping_priority_over_similar() {
        let temp_dir = std::env::temp_dir();
        let explicit_path = temp_dir.join("explicit_kai.ttf");
        let similar_path = temp_dir.join("similar_kai.ttf");
        std::fs::write(&explicit_path, b"explicit").unwrap();
        std::fs::write(&similar_path, b"similar").unwrap();

        let mut loader = FontLoader::new();
        // 直接注册 "KaiTi"
        loader.register_font("KaiTi", &explicit_path);
        // 注册相似替换目标
        loader.register_font("楷体", &similar_path);
        loader.add_similar_font_replace_regex_mapping(".*Kai.*", "楷体");

        // 直接查找应优先于相似替换
        let found = loader.find_font_with_similar_replace(None, Some("KaiTi"));
        assert!(found.is_some());
        assert_eq!(found.as_ref().unwrap(), &explicit_path);

        let _ = std::fs::remove_file(&explicit_path);
        let _ = std::fs::remove_file(&similar_path);
    }

    #[test]
    fn test_with_system_dirs_has_default_rules() {
        let loader = FontLoader::with_system_dirs();
        // 默认应有 4 条相似替换规则（去重后）
        assert!(loader.similar_replace_count() >= 4);
        // 默认应有别名映射
        assert!(!loader.alias_map().is_empty());
    }

    #[test]
    fn test_empty_pattern_ignored() {
        let mut loader = FontLoader::new();
        loader.add_similar_font_replace_regex_mapping("", "target");
        loader.add_similar_font_replace_regex_mapping("pattern", "");
        assert_eq!(loader.similar_replace_count(), 0);
    }

    // ── 模式匹配器单元测试 ──

    #[test]
    fn test_similar_pattern_java_regex() {
        let p = SimilarPattern::from_pattern(".*Kai.*");
        assert!(p.is_match("KaiTi"));
        assert!(p.is_match("STKaiti"));
        assert!(!p.is_match("SimSun"));
    }

    #[test]
    fn test_similar_pattern_glob() {
        let p = SimilarPattern::from_pattern("*Song*");
        assert!(p.is_match("Songti SC"));
        assert!(p.is_match("FZSongKeBen"));
        assert!(!p.is_match("KaiTi"));
    }

    #[test]
    fn test_similar_pattern_prefix() {
        let p = SimilarPattern::from_pattern("Sim*");
        assert!(p.is_match("SimSun"));
        assert!(p.is_match("Simple"));
        assert!(!p.is_match("KaiSim"));
    }

    #[test]
    fn test_similar_pattern_suffix() {
        let p = SimilarPattern::from_pattern("*Sun");
        assert!(p.is_match("SimSun"));
        assert!(!p.is_match("SunSim"));
    }

    #[test]
    fn test_similar_pattern_exact() {
        let p = SimilarPattern::from_pattern("Arial");
        assert!(p.is_match("Arial"));
        assert!(!p.is_match("ArialBold"));
    }
}
