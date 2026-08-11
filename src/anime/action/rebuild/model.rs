use std::collections::BTreeMap;

use crate::anime::action::matcher::ExtraKind;

#[derive(Debug, Clone)]
pub(crate) struct ExtraItem {
    pub(crate) kind: ExtraKind,
    pub(crate) title: String,
    pub(crate) tag: Option<String>,
}

/// A parsed file node; `id` is its rid in the raw tree so it can be
/// moved later without losing any of its fields.
#[derive(Debug, Clone)]
pub(crate) struct ParsedFile {
    pub(crate) id: u64,
    /// False only for non-file nodes, which must never get a season.
    pub(crate) is_file: bool,
    pub(crate) season: Option<u32>,
    pub(crate) episode: Option<u32>,
    /// Nearest regular (non-extra) ancestor directory; used to anchor
    /// season-less extras to the season of sibling main-episode files.
    pub(crate) anchor_parent: Option<u64>,
    pub(crate) extra: Option<ExtraItem>,
}

/// Files belonging to one season: those with a recognized episode
/// number, plus those without one (kept under the season's `unknown`).
#[derive(Debug, Default)]
pub(crate) struct ParsedSeason {
    pub(crate) episodes: Vec<ParsedFile>,
    pub(crate) unknown: Vec<ParsedFile>,
    /// Extra files found loose inside this season (no extra directory).
    pub(crate) loose: Vec<ParsedFile>,
}

/// An extra directory discovered during the walk. The tree is stored flat
/// with `parent`/`children` indices; `files` are the extras directly
/// inside. `season` is inherited from the directory chain or filled in by
/// anchoring; `None` means the whole subtree lands at series level.
#[derive(Debug)]
pub(crate) struct ExtraDir {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) kind: ExtraKind,
    pub(crate) season: Option<u32>,
    pub(crate) anchor_parent: Option<u64>,
    pub(crate) parent: Option<usize>,
    pub(crate) children: Vec<usize>,
    pub(crate) files: Vec<ParsedFile>,
}

/// Fully parsed series: seasons → files, plus files that could not be
/// placed, plus every raw directory node that must be deleted.
#[derive(Debug, Default)]
pub(crate) struct ParsedSeries {
    pub(crate) name: String,
    pub(crate) seasons: BTreeMap<u32, ParsedSeason>,
    pub(crate) unknown: Vec<ParsedFile>,
    /// DFS pre-order; delete in reverse (deepest first).
    pub(crate) dirs_to_delete: Vec<u64>,
    /// Flat list of every extra directory in the series.
    pub(crate) extra_dirs: Vec<ExtraDir>,
    /// Season-less extra files outside any extra directory.
    pub(crate) loose_extras: Vec<ParsedFile>,
    /// Regular directory → season of its main-episode children (anchoring).
    pub(crate) dir_seasons: BTreeMap<u64, u32>,
}
