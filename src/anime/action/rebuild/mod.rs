pub(crate) mod model;
pub(crate) mod rules;
pub(crate) mod tree;
pub(crate) mod walk;

use cosmox_api::api::bindings::cosmox::plugin::context::MetadataQuery;
use cosmox_api::handle::{MetadataView, PathMappingView};
use cosmox_api::metadata::MetadataType;

use crate::anime::action::rebuild::tree::rebuild_series;
use crate::anime::action::rebuild::walk::{merge_series, parse_series};

/// Rebuild every top-level directory into the canonical tree structure.
///
/// Phase 1 parses all top-level nodes read-only. Phase 2 merges same-title
/// series, moves unplaceable top-level files under `unknown`, rebuilds each
/// series, and deletes orphaned virtual shells from previous runs.
///
/// Every file node that survives the rebuild is pushed into `path_mapping`
/// (`data_file_map_id`), since the host leaves the mapping empty by default.
pub fn rebuild_metadata_tree(view: MetadataView, path_mapping: PathMappingView) {
    let root_query = MetadataQuery::Id(0);
    let roots = view.children(&root_query);
    log::info!("rebuild metadata tree: {} root nodes", roots.len());

    // Phase 1: parse everything (read-only).
    let mut series_list = Vec::new();
    let mut top_files = Vec::new();
    let mut orphan_virtuals = Vec::new();
    for child in roots {
        let key = MetadataQuery::Id(child);
        let mtype = view.metadata_type(&key);
        match mtype {
            Some(MetadataType::Directory) => series_list.push(parse_series(view.clone(), child)),
            // Leftover virtual shells from a previous run: drop them instead
            // of moving them into the top-level `unknown`.
            Some(MetadataType::Virtual) => orphan_virtuals.push(child),
            _ => top_files.push(child),
        }
    }
    log::info!(
        "parsed {} series, {} top-level files, {} orphan virtuals",
        series_list.len(),
        top_files.len(),
        orphan_virtuals.len()
    );

    // Phase 2: rebuild.
    let series_list = merge_series(series_list);
    if !top_files.is_empty()
        && let Some(unknown_key) = view
            .create_node(&root_query, "unknown", MetadataType::Virtual)
            .map(MetadataQuery::Id)
    {
        log::info!("moving {} top-level files under unknown", top_files.len());
        for id in top_files {
            view.move_node(&MetadataQuery::Id(id), &unknown_key);
        }
    }
    for series in &series_list {
        rebuild_series(view.clone(), &path_mapping, &root_query, series);
    }
    for rid in orphan_virtuals {
        view.delete(&MetadataQuery::Id(rid));
    }
    log::info!(
        "rebuild metadata tree finished ({} series)",
        series_list.len()
    );
}
