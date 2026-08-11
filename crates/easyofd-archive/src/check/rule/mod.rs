//! OFD-A 合规规则实现。
//!
//! 对应 Java: org.ofdrw.archive.check.rule

#![allow(clippy::case_sensitive_file_extension_comparisons)]

pub mod annotation_rule;
pub mod attachment_rule;
pub mod audio_video_rule;
pub mod clip_area_rule;
pub mod color_profile_rule;
pub mod color_space_rule;
pub mod extension_rule;
pub mod external_resource_rule;
pub mod font_subset_rule;
pub mod image_extension_rule;
pub mod image_format_rule;
pub mod image_interpolate_rule;
pub mod image_resource_reg_rule;
pub mod non_goto_action_rule;
pub mod outline_action_rule;
pub mod page_block_depth_rule;
pub mod permission_rule;
pub mod resource_placement_rule;
pub mod single_doc_rule;
pub mod text_hscale_rule;
pub mod text_size_rule;

pub use annotation_rule::AnnotationRule;
pub use attachment_rule::AttachmentRule;
pub use audio_video_rule::AudioVideoRule;
pub use clip_area_rule::ClipAreaRule;
pub use color_profile_rule::ColorProfileRule;
pub use color_space_rule::ColorSpaceRule;
pub use extension_rule::ExtensionRule;
pub use external_resource_rule::ExternalResourceRule;
pub use font_subset_rule::FontSubsetRule;
pub use image_extension_rule::ImageExtensionRule;
pub use image_format_rule::ImageFormatRule;
pub use image_interpolate_rule::ImageInterpolateRule;
pub use image_resource_reg_rule::ImageResourceRegRule;
pub use non_goto_action_rule::NonGotoActionRule;
pub use outline_action_rule::OutlineActionRule;
pub use page_block_depth_rule::PageBlockDepthRule;
pub use permission_rule::PermissionRule;
pub use resource_placement_rule::ResourcePlacementRule;
pub use single_doc_rule::SingleDocRule;
pub use text_hscale_rule::TextHScaleRule;
pub use text_size_rule::TextSizeRule;
