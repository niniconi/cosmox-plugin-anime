use std::collections::{BTreeMap, BTreeSet};

use cosmox_api::api::bindings::cosmox::plugin::context::MetadataQuery;
use cosmox_api::handle::{MetadataView, PathMappingView};
use cosmox_api::metadata::MetadataType;

use crate::anime::action::matcher::{ExtraKind, extra_bucket_name, extra_kind_str};
use crate::anime::action::rebuild::model::{ExtraItem, ParsedFile, ParsedSeries};
use crate::anime::define::Episode;

/// Read the node's `file:` url and push it into `data_file_map_id`, so the
/// host can resolve the file through path_mapping.
fn push_data_file(view: MetadataView, path_mapping: &PathMappingView, id: u64) {
    let Some(meta) = view.query::<()>(&MetadataQuery::Id(id)) else {
        return;
    };
    if meta.url.is_empty() {
        return;
    }
    if let Err(err) = path_mapping.push(id, "data_file_map_id", &meta.url) {
        log::warn!("push data file mapping for {id} failed: {err:?}");
    }
}

/// Write the parsed season/episode numbers into the node's `extend` map.
fn annotate_file(view: MetadataView, id: u64, season: Option<u32>, episode: Option<u32>) {
    let key = MetadataQuery::Id(id);
    view.write_extend(
        &key,
        Episode {
            season_number: season,
            episode_number: episode,
            ..Default::default()
        },
    );
}

fn annotate_extra(view: MetadataView, id: u64, extra: &ExtraItem, season: Option<u32>) {
    let key = MetadataQuery::Id(id);
    view.write_extend(
        &key,
        Episode {
            season_number: season,
            extra_kind: Some(extra_kind_str(extra.kind).to_string()),
            extra_title: (!extra.title.is_empty()).then(|| extra.title.clone()),
            extra_tag: extra.tag.clone(),
            ..Default::default()
        },
    );
}

fn annotate_extra_dir(view: MetadataView, key: &MetadataQuery, kind: ExtraKind) {
    view.write_extend(
        key,
        Episode {
            extra_kind: Some(extra_kind_str(kind).to_string()),
            ..Default::default()
        },
    );
}

/// Whether an extra directory should be flattened into its bucket: names
/// equal up to pluralization (`Scan`/`Scans`, `EXTRA`/`Extras`) collapse.
fn same_bucket_name(dir_name: &str, bucket: &str) -> bool {
    let a = dir_name.to_lowercase();
    let b = bucket.to_lowercase();
    a.trim_end_matches('s') == b.trim_end_matches('s')
}

fn move_extra_file(
    view: MetadataView,
    path_mapping: &PathMappingView,
    parent: &MetadataQuery,
    file: &ParsedFile,
) {
    view.move_node(&MetadataQuery::Id(file.id), parent);
    if let Some(extra) = &file.extra {
        annotate_extra(view.clone(), file.id, extra, file.season);
    }
    push_data_file(view, path_mapping, file.id);
}

/// Rebuild one extra directory under `key`, preserving its original
/// hierarchy: children keep their names, files stay put when their kind
/// matches the directory's, otherwise they are grouped into kind buckets.
fn rebuild_extra_dir(
    view: MetadataView,
    path_mapping: &PathMappingView,
    key: &MetadataQuery,
    series: &ParsedSeries,
    idx: usize,
) {
    let dir = &series.extra_dirs[idx];
    for file in &dir.files {
        if let Some(extra) = &file.extra
            && extra.kind == dir.kind
        {
            move_extra_file(view.clone(), path_mapping, key, file);
        }
    }
    let mut buckets: BTreeMap<ExtraKind, Vec<&ParsedFile>> = BTreeMap::new();
    for file in &dir.files {
        if let Some(extra) = &file.extra
            && extra.kind != dir.kind
        {
            buckets.entry(extra.kind).or_default().push(file);
        }
    }
    for (kind, files) in buckets {
        let Some(bucket_key) = view
            .create_node(key, extra_bucket_name(kind), MetadataType::Virtual)
            .map(MetadataQuery::Id)
        else {
            continue;
        };
        annotate_extra_dir(view.clone(), &bucket_key, kind);
        for file in files {
            move_extra_file(view.clone(), path_mapping, &bucket_key, file);
        }
    }
    for &child in &dir.children {
        let child_dir = &series.extra_dirs[child];
        let Some(child_key) = view
            .create_node(key, &child_dir.name, MetadataType::Virtual)
            .map(MetadataQuery::Id)
        else {
            continue;
        };
        annotate_extra_dir(view.clone(), &child_key, child_dir.kind);
        rebuild_extra_dir(view.clone(), path_mapping, &child_key, series, child);
    }
}

/// Rebuild the extras tree under `parent`: top-level entries are grouped
/// into kind buckets; a directory whose name matches its bucket is
/// flattened so `Scans/` keeps the shape the release shipped with.
fn rebuild_extras(
    view: MetadataView,
    path_mapping: &PathMappingView,
    parent: &MetadataQuery,
    series: &ParsedSeries,
    roots: &[usize],
    loose: &[&ParsedFile],
) {
    let mut buckets: BTreeMap<ExtraKind, (Vec<usize>, Vec<&ParsedFile>)> = BTreeMap::new();
    for &idx in roots {
        buckets
            .entry(series.extra_dirs[idx].kind)
            .or_default()
            .0
            .push(idx);
    }
    for file in loose {
        if let Some(extra) = &file.extra {
            buckets.entry(extra.kind).or_default().1.push(file);
        }
    }
    for (kind, (dirs, files)) in buckets {
        let Some(bucket_key) = view
            .create_node(parent, extra_bucket_name(kind), MetadataType::Virtual)
            .map(MetadataQuery::Id)
        else {
            continue;
        };
        annotate_extra_dir(view.clone(), &bucket_key, kind);
        for file in files {
            move_extra_file(view.clone(), path_mapping, &bucket_key, file);
        }
        for idx in dirs {
            let dir = &series.extra_dirs[idx];
            if same_bucket_name(&dir.name, extra_bucket_name(kind)) {
                rebuild_extra_dir(view.clone(), path_mapping, &bucket_key, series, idx);
            } else {
                let Some(dir_key) = view
                    .create_node(&bucket_key, &dir.name, MetadataType::Virtual)
                    .map(MetadataQuery::Id)
                else {
                    continue;
                };
                annotate_extra_dir(view.clone(), &dir_key, dir.kind);
                rebuild_extra_dir(view.clone(), path_mapping, &dir_key, series, idx);
            }
        }
    }
}

/// DFS pre-order of extra-directory ids for deletion (reverse order).
fn collect_extra_ids(series: &ParsedSeries, roots: &[usize], out: &mut Vec<u64>) {
    for &idx in roots {
        out.push(series.extra_dirs[idx].id);
        for &child in &series.extra_dirs[idx].children {
            collect_extra_ids(series, &[child], out);
        }
    }
}

/// Rebuild one series.
///
/// 1. Create a virtual series node under the root;
/// 2. create a virtual `Season N` node per season and move files into it,
///    with season members lacking an episode number under a nested `unknown`;
/// 3. rebuild extra trees under each season's `extras`, and the series-level
///    `extras` for anything that could not be anchored;
/// 4. move unplaceable files under a virtual `unknown` node;
/// 5. delete every raw directory node (deepest first).
pub(crate) fn rebuild_series(
    view: MetadataView,
    path_mapping: &PathMappingView,
    root_query: &MetadataQuery,
    series: &ParsedSeries,
) {
    let season_count = series.seasons.len();
    log::info!(
        "rebuild series '{}' ({} seasons)",
        series.name,
        season_count
    );
    let Some(series_key) = view
        .create_node(root_query, &series.name, MetadataType::Virtual)
        .map(MetadataQuery::Id)
    else {
        return;
    };

    // Split top-level extra dirs by season (inherited or anchored).
    let mut season_roots: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    let mut series_roots: Vec<usize> = Vec::new();
    for (idx, dir) in series.extra_dirs.iter().enumerate() {
        if dir.parent.is_none() {
            match dir.season {
                Some(s) => season_roots.entry(s).or_default().push(idx),
                None => series_roots.push(idx),
            }
        }
    }

    // A season may exist with extras only and no main episodes.
    let mut season_nums: BTreeSet<u32> = series.seasons.keys().copied().collect();
    season_nums.extend(season_roots.keys().copied());

    let mut extra_dir_ids: Vec<u64> = Vec::new();
    for season_num in season_nums {
        let Some(season_key) = view
            .create_node(
                &series_key,
                &format!("Season {season_num}"),
                MetadataType::Virtual,
            )
            .map(MetadataQuery::Id)
        else {
            continue;
        };
        if let Some(bucket) = series.seasons.get(&season_num) {
            for f in &bucket.episodes {
                view.move_node(&MetadataQuery::Id(f.id), &season_key);
                annotate_file(view.clone(), f.id, Some(season_num), f.episode);
                push_data_file(view.clone(), path_mapping, f.id);
            }
            if !bucket.unknown.is_empty()
                && let Some(unknown_key) = view
                    .create_node(&season_key, "unknown", MetadataType::Virtual)
                    .map(MetadataQuery::Id)
            {
                for f in &bucket.unknown {
                    view.move_node(&MetadataQuery::Id(f.id), &unknown_key);
                    annotate_file(view.clone(), f.id, Some(season_num), f.episode);
                    push_data_file(view.clone(), path_mapping, f.id);
                }
            }
        }
        let roots = season_roots.remove(&season_num).unwrap_or_default();
        let loose: Vec<&ParsedFile> = series
            .seasons
            .get(&season_num)
            .map(|b| b.loose.iter().collect())
            .unwrap_or_default();
        if (!roots.is_empty() || !loose.is_empty())
            && let Some(extras_key) = view
                .create_node(&season_key, "extras", MetadataType::Virtual)
                .map(MetadataQuery::Id)
        {
            rebuild_extras(view.clone(), path_mapping, &extras_key, series, &roots, &loose);
            collect_extra_ids(series, &roots, &mut extra_dir_ids);
        }
    }

    if (!series_roots.is_empty() || !series.loose_extras.is_empty())
        && let Some(extras_key) = view
            .create_node(&series_key, "extras", MetadataType::Virtual)
            .map(MetadataQuery::Id)
    {
        let loose: Vec<&ParsedFile> = series.loose_extras.iter().collect();
        rebuild_extras(view.clone(), path_mapping, &extras_key, series, &series_roots, &loose);
        collect_extra_ids(series, &series_roots, &mut extra_dir_ids);
    }

    if !series.unknown.is_empty()
        && let Some(unknown_key) = view
            .create_node(&series_key, "unknown", MetadataType::Virtual)
            .map(MetadataQuery::Id)
    {
        for f in &series.unknown {
            view.move_node(&MetadataQuery::Id(f.id), &unknown_key);
            annotate_file(view.clone(), f.id, f.season, f.episode);
            push_data_file(view.clone(), path_mapping, f.id);
        }
    }

    // Extra directories are deeper than any regular directory they sit in;
    // delete them first so a cascading `delete` never hits them twice.
    for &dir_id in extra_dir_ids.iter().rev() {
        view.delete(&MetadataQuery::Id(dir_id));
    }
    for &dir_id in series.dirs_to_delete.iter().rev() {
        view.delete(&MetadataQuery::Id(dir_id));
    }
    let file_count: usize = series
        .seasons
        .values()
        .map(|s| s.episodes.len() + s.unknown.len() + s.loose.len())
        .sum();
    log::info!(
        "series '{}' rebuilt ({} seasons, {} files, {} extras deleted, {} raw dirs deleted)",
        series.name,
        season_count,
        file_count,
        extra_dir_ids.len(),
        series.dirs_to_delete.len()
    );
}
