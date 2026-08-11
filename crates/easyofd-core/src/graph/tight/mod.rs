//! 紧凑图形子包 (graph.tight)。
//!
//! 对应 Java: org.ofdrw.core.graph.tight
//! 包含路径方法类型（Arc、Line、Close、CubicBezier、QuadraticBezier、Move）

pub mod method;

pub use method::{
    Arc as ArcCommand, Close as CloseCommand, CubicBezier, Line as LineCommand,
    Move as MoveCommand, QuadraticBezier,
};
