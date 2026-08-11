use std::collections::BTreeMap;

use cosmox_api::api::bindings::cosmox::plugin::context::MetadataQuery;
use cosmox_api::handle::MetadataView;
use cosmox_api::metadata::MetadataType;

use crate::anime::action::matcher::{
    ParsedInfo, Scored, TokenKind, extra_tag_from_name, extra_title_from_name, parse_info,
    series_title_from_name, strip_season_markers,
};
use crate::anime::action::rebuild::model::{ExtraDir, ExtraItem, ParsedFile, ParsedSeries};
use crate::anime::action::rebuild::rules::{DIR_RULES, FILE_RULES};

/// Recursively read-only walk one series subtree, collecting files by
/// season, recording all directories for later deletion, and carving out
/// extra directories (which are rebuilt instead of deleted). The series
/// root never becomes an extra directory: release-group names carry extra
/// labels (`[01-13TV全集+特典映像]`, `[BD ...]`) yet hold the main files.
fn walk(
    view: MetadataView,
    node_id: u64,
    inherited: &ParsedInfo,
    anchor_parent: Option<u64>,
    extra_dir: Option<usize>,
    is_root: bool,
    series: &mut ParsedSeries,
) {
    let key = MetadataQuery::Id(node_id);

    let Some(mtype) = view.metadata_type(&key) else {
        return;
    };

    let Some(name) = view.name(&key) else {
        return;
    };

    match mtype {
        MetadataType::File => {
            let own = parse_info(&name, FILE_RULES);
            let effective = ParsedInfo::inherit(inherited, &own);
            if let Some(Scored {
                value: TokenKind::Extra(kind),
                ..
            }) = effective.kind
            {
                let file = ParsedFile {
                    id: node_id,
                    is_file: true,
                    season: effective.season.as_ref().map(|s| s.value),
                    episode: None,
                    anchor_parent,
                    extra: Some(ExtraItem {
                        kind,
                        title: extra_title_from_name(&name),
                        tag: extra_tag_from_name(&name, kind),
                    }),
                };
                if let Some(dir) = extra_dir {
                    series.extra_dirs[dir].files.push(file);
                } else if let Some(s) = file.season {
                    series.seasons.entry(s).or_default().loose.push(file);
                } else {
                    series.loose_extras.push(file);
                }
                return;
            }
            if let Some(pid) = anchor_parent
                && let Some(s) = effective.season.as_ref().map(|s| s.value)
            {
                series.dir_seasons.entry(pid).or_insert(s);
            }
            let file = ParsedFile {
                id: node_id,
                is_file: true,
                season: effective.season.as_ref().map(|s| s.value),
                episode: effective.episode.as_ref().map(|e| e.value),
                anchor_parent,
                extra: None,
            };
            match file.season {
                Some(season) => {
                    let bucket = series.seasons.entry(season).or_default();
                    if file.episode.is_some() {
                        bucket.episodes.push(file);
                    } else {
                        bucket.unknown.push(file);
                    }
                }
                None => series.unknown.push(file),
            }
        }
        // Raw directories and leftover virtual nodes from a previous run
        // are rebuilt away; their children (files) are re-collected first.
        // Extra directories (themselves or via inheritance) are preserved
        // as `ExtraDir` trees instead of being flattened.
        MetadataType::Directory | MetadataType::Virtual => {
            let own = parse_info(&name, DIR_RULES);
            let mut effective = ParsedInfo::inherit(inherited, &own);
            // The series root's name is a release-group name, not content:
            // it carries labels like `[01-13TV全集+特典映像]` / `+SP` that
            // match `match_extra_generic`. If that kind leaked to the whole
            // subtree, every main-episode file would be classified as a
            // Generic extra and dumped into `extras/Extras`.
            if is_root {
                effective.kind = None;
            }
            if !is_root
                && let Some(Scored {
                    value: TokenKind::Extra(kind),
                    ..
                }) = effective.kind
            {
                let idx = series.extra_dirs.len();
                series.extra_dirs.push(ExtraDir {
                    id: node_id,
                    name,
                    kind,
                    season: effective.season.as_ref().map(|s| s.value),
                    anchor_parent,
                    parent: extra_dir,
                    children: Vec::new(),
                    files: Vec::new(),
                });
                if let Some(parent) = extra_dir {
                    series.extra_dirs[parent].children.push(idx);
                }
                for child in view.children(&key) {
                    walk(
                        view.clone(),
                        child,
                        &effective,
                        anchor_parent,
                        Some(idx),
                        false,
                        series,
                    );
                }
            } else {
                series.dirs_to_delete.push(node_id);
                for child in view.children(&key) {
                    walk(
                        view.clone(),
                        child,
                        &effective,
                        Some(node_id),
                        extra_dir,
                        false,
                        series,
                    );
                }
            }
        }
        // Other node types are preserved whole (with their subtree)
        // under the series' `unknown` bucket.
        _ => {
            series.unknown.push(ParsedFile {
                id: node_id,
                is_file: false,
                season: None,
                episode: None,
                anchor_parent: None,
                extra: None,
            });
        }
    }
}

/// Season-less extras adopt the season of main-episode files sitting in
/// the same regular directory; the rest stay at series level.
fn anchor_extras(series: &mut ParsedSeries) {
    for dir in &mut series.extra_dirs {
        if dir.season.is_none()
            && let Some(pid) = dir.anchor_parent
        {
            dir.season = series.dir_seasons.get(&pid).copied();
        }
    }
    let mut kept = Vec::new();
    for file in series.loose_extras.drain(..) {
        let season = file
            .anchor_parent
            .and_then(|pid| series.dir_seasons.get(&pid).copied());
        if let Some(s) = season {
            series.seasons.entry(s).or_default().loose.push(file);
        } else {
            kept.push(file);
        }
    }
    series.loose_extras = kept;
}

/// Files whose season could not be recognized are assigned to the
/// first season by default. Non-file nodes are left in `unknown`.
fn assign_unknown_to_first_season(series: &mut ParsedSeries) {
    if !series.unknown.iter().any(|f| f.is_file) {
        return;
    }
    let first = series.seasons.entry(1).or_default();
    let mut kept = Vec::new();
    for f in series.unknown.drain(..) {
        if f.is_file {
            if f.episode.is_some() {
                first.episodes.push(f);
            } else {
                first.unknown.push(f);
            }
        } else {
            kept.push(f);
        }
    }
    series.unknown = kept;
}

/// Walk the series subtree rooted at `root_id` and collect everything the
/// rebuild needs: files grouped by season, extra trees, and the list of
/// directories to delete.
pub(crate) fn parse_series(view: MetadataView, root_id: u64) -> ParsedSeries {
    let root_key = MetadataQuery::Id(root_id);

    let name = view.name(&root_key).unwrap_or_default();
    let cleaned = series_title_from_name(&name);
    // Strip season markers now so merge keys on the pure title; the season
    // number itself is already inherited into files by `walk` via the
    // root's DIR_RULES classification, so it is intentionally discarded.
    let title = strip_season_markers(&cleaned);
    let display_name = if title.is_empty() { name } else { title };

    let mut series = ParsedSeries {
        name: display_name,
        ..Default::default()
    };
    walk(
        view,
        root_id,
        &ParsedInfo::default(),
        None,
        None,
        true,
        &mut series,
    );
    anchor_extras(&mut series);
    assign_unknown_to_first_season(&mut series);
    let season_count = series.seasons.len();
    let file_count: usize = series
        .seasons
        .values()
        .map(|s| s.episodes.len() + s.unknown.len() + s.loose.len())
        .sum();
    log::debug!(
        "parsed series '{}': {} seasons, {} files, {} extra dirs, {} unknown",
        series.name,
        season_count,
        file_count,
        series.extra_dirs.len(),
        series.unknown.len()
    );
    series
}

/// Merge series whose names are equal after stripping season markers,
/// e.g. `葬送的芙莉蓮` + `葬送的芙莉蓮 S2`. The markers are stripped
/// during `parse_series`, so `series.name` is already a pure title and
/// the merge keys on it directly. Works uniformly for any number of
/// candidates, including a single `xxx 第二季` directory with no other
/// seasons present.
pub(crate) fn merge_series(list: Vec<ParsedSeries>) -> Vec<ParsedSeries> {
    let mut merged: Vec<ParsedSeries> = Vec::new();
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    for series in list {
        if let Some(&i) = index.get(&series.name) {
            let target = &mut merged[i];
            log::debug!(
                "merge '{}' into existing series ({} seasons)",
                series.name,
                series.seasons.len()
            );
            for (season, bucket) in series.seasons {
                let tb = target.seasons.entry(season).or_default();
                tb.episodes.extend(bucket.episodes);
                tb.unknown.extend(bucket.unknown);
                tb.loose.extend(bucket.loose);
            }
            target.unknown.extend(series.unknown);
            target.loose_extras.extend(series.loose_extras);
            target.dirs_to_delete.extend(series.dirs_to_delete);
            // Extra-dir indices shift by the target's current length.
            let offset = target.extra_dirs.len();
            for mut dir in series.extra_dirs {
                dir.parent = dir.parent.map(|p| p + offset);
                for c in &mut dir.children {
                    *c += offset;
                }
                target.extra_dirs.push(dir);
            }
        } else {
            index.insert(series.name.clone(), merged.len());
            merged.push(series);
        }
    }
    merged
}
