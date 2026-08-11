//! 印章位置定义模块。
//!
//! 对应 Java: `org.ofdrw.sign.stamppos`
//!
//! 提供印章放置位置的类型定义：
//! - [`Side`]：骑缝章所在边
//! - [`NormalStampPos`]：普通印章位置
//! - [`RidingStampPos`]：骑缝章位置
//! - [`CuttingRatio`]：骑缝章切割比例

mod cutting_ratio;
mod cutting_ride_stamp_pos;
mod normal_stamp_pos;
mod riding_stamp_pos;
mod side;

pub use cutting_ratio::CuttingRatio;
pub use cutting_ride_stamp_pos::CuttingRideStampPos;
pub use normal_stamp_pos::NormalStampPos;
pub use riding_stamp_pos::RidingStampPos;
pub use side::Side;
