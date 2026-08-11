//! 基本数据类型 (GB/T 33190 附录A)
//!
//! 对应 Java: org.ofdrw.core.basicType

#![allow(non_camel_case_types)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::float_cmp)]

pub mod opt_val;
pub mod st_array;
pub mod st_box;
pub mod st_id;
pub mod st_loc;
pub mod st_pos;
pub mod st_ref_id;

pub use opt_val::OptVal;
pub use st_array::ST_Array;
pub use st_box::ST_Box;
pub use st_id::ST_ID;
pub use st_loc::ST_Loc;
pub use st_pos::ST_Pos;
pub use st_ref_id::ST_RefID;
