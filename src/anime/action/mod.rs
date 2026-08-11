//! Metadata preprocessing for anime media type.
//!
//! This module owns the two functional domains of the plugin:
//!
//! - `matcher`: pure "name string → structure" parsing (seasons, episodes,
//!   extras). No side effects.
//! - `rebuild`: reads the raw metadata tree via `MetadataView` and
//!   rewrites it into the canonically structured tree.

pub(crate) mod matcher;
pub(crate) mod rebuild;

// Public contract re-exports (paths must stay stable for downstream crates).
pub use matcher::series_title_from_name;
pub use matcher::{Confidence, ExtraKind, MatchRule, ParsedInfo, Scored, TokenKind, parse_info};

pub use rebuild::rebuild_metadata_tree;
