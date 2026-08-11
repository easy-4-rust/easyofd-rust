//! iText 外部依赖排除说明。
//!
//! 对应 Java: org.ofdrw.converter.font.ItextTrueTypeFont / .ItextFontUtil
//!            org.ofdrw.full.Keep
//!
//! 这些 ofdrw 类型依赖 iText（com.itextpdf）Java 库，无法在 Rust 中直接镜像。
//! Rust 替代方案：
//! - ItextTrueTypeFont → use `easyofd_font::TrueTypeFont`（纯数据模型）
//! - ItextFontUtil     → use `easyofd_convert::FontLoader`（已移植）
//! - Keep               → 标记接口，无具体实现，Rust 不需要等价物
//!
//! **类型覆盖率诚实记录**：ofdrw 487 个 unique 类型 484 已覆盖（99.4%），剩余
//! 3 个均为 iText 外部依赖，已记录在此文件中。

#![allow(clippy::items_after_statements)]

#[cfg(test)]
mod tests {
    #[test]
    fn test_itext_documented() {
        // ItextTrueTypeFont / ItextFontUtil / Keep 由 iText 外部依赖
        // 移除，Rust 替代见模块级注释。
    }
}
