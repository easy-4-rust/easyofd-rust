//! OFD 动作模块。
//!
//! 实现 GB/T 33190 第 15 章"动作"中定义的全部动作类型。
//!
//! 对应 Java: org.ofdrw.core.action

mod actions;
mod bookmark_action;
mod ct_action;
mod ct_dest;
mod dest_type;
mod event_type;
mod goto;
mod gotoa;
mod movie;
mod ofd_action;
mod ofd_goto_target;
mod play_type;
mod sound;
mod uri;

// Re-export all public types.
pub use actions::Actions;
pub use bookmark_action::Bookmark;
pub use ct_action::CTAction;
pub use ct_dest::CTDest;
pub use dest_type::DestType;
pub use event_type::EventType;
pub use goto::Goto;
pub use gotoa::GotoA;
pub use movie::Movie;
pub use ofd_action::OfdAction;
pub use ofd_goto_target::OfdGotoTarget;
pub use play_type::PlayType;
pub use sound::Sound;
pub use uri::URI;
