//! 文本子包 (text)。
//!
//! 对应 GB/T 33190-2016 第 11 节，包含文本对象、字体描述、
//! 文字定位和字形变换等类型。

mod ct_cg_transform;
mod ct_font;
mod ct_text;
mod text_code;

pub use ct_cg_transform::CT_CGTransform;
pub use ct_font::CT_Font;
pub use ct_text::{CT_Text, Direction};
pub use text_code::TextCode;
