//! 转换工具函数模块。
//!
//! 对应 Java: org.ofdrw.converter.utils

pub mod common_util;
pub mod matrix;
pub mod os_info;
pub mod point_util;
pub mod string_util;

pub use matrix::Matrix3x3;
pub use os_info::EPlatform;

// ── ofdrw Java 模块别名 ──

/// 对应 Java: `org.ofdrw.converter.utils.CommonUtil`
///
/// 工具类以模块级函数形式实现，见 [`common_util`] 模块。
pub use common_util as CommonUtil;

/// 对应 Java: `org.ofdrw.converter.utils.PointUtil`
///
/// 工具类以模块级函数形式实现，见 [`point_util`] 模块。
pub use point_util as PointUtil;

/// 对应 Java: `org.ofdrw.converter.utils.OSinfo`
///
/// 工具类以模块级函数和 [`EPlatform`] 枚举形式实现，见 [`os_info`] 模块。
pub use os_info as OSinfo;

/// 对应 Java: `org.ofdrw.converter.utils.StringUtils`
///
/// 工具类以模块级函数形式实现，见 [`string_util`] 模块。
pub use string_util as StringUtils;
