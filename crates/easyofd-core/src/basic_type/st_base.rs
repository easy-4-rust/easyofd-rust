//! ST_Base 基本类型基类。
//!
//! 对应 Java: org.ofdrw.core.basicType.STBase
//!
//! 所有 OFD 基本类型（ST_Array、ST_Box、ST_ID 等）的公共特征。

/// OFD 基本类型公共特征。
///
/// 对应 Java: org.ofdrw.core.basicType.STBase
///
/// 在 Java 版中 `STBase` 是所有 ST_ 类型的基类，定义了
/// `toString()` 和 XML 序列化行为。Rust 版用 trait 实现等价功能。
pub trait STBase {
    /// 转为 OFD XML 属性值字符串。
    fn to_xml_string(&self) -> String;
}

/// 裁剪区域特征。
///
/// 对应 Java: org.ofdrw.core.pageDescription.clips.ClipAble
///
/// 可被裁剪的图元类型实现此 trait，表示支持裁剪区域设置。
pub trait ClipAble {
    /// 设置裁剪区域。
    fn set_clip(&mut self, clip: crate::page_description::clips::CT_Clip);

    /// 获取裁剪区域引用。
    fn clip(&self) -> Option<&crate::page_description::clips::CT_Clip>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // STBase trait 不能直接实例化，测试通过具体类型验证
    #[test]
    fn test_st_base_trait_exists() {
        // 验证 trait 可以被引用
        fn _assert_st_base<T: STBase>() {}
        fn _assert_clip_able<T: ClipAble>() {}
    }
}
