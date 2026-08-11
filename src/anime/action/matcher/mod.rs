//! Pure "name string → structure" parsing. No `MetadataView` dependency,
//! no side effects.

pub mod core;
pub mod episode;
pub mod ext;
pub mod extra;
pub mod name;
pub mod season;

// Public contract items (re-exported further by `anime::action`).
pub use core::{Confidence, ExtraKind, MatchRule, ParsedInfo, Scored, TokenKind, parse_info};
pub use name::series_title_from_name;

// Items consumed by the rebuild layer (crate-internal).
pub(crate) use episode::{
    match_episode_bracketed, match_episode_chinese, match_episode_dash_number, match_episode_tv,
};
pub(crate) use extra::{
    extra_bucket_name, extra_kind_str, extra_tag_from_name, extra_title_from_name,
    match_extra_audio_commentary, match_extra_bonus_disc, match_extra_ending, match_extra_font,
    match_extra_generic, match_extra_live, match_extra_menu, match_extra_next_preview,
    match_extra_opening, match_extra_promotion, match_extra_scan,
};
pub(crate) use season::{
    match_season_chinese, match_season_english, match_season_numbered_prefix,
    match_season_numbered_prefix_any, match_season_part, match_season_part_any,
    match_season_s_number, strip_season_markers,
};
