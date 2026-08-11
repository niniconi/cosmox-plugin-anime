#[cfg(feature = "plugin")]
use cosmox_api::api::plugin;

pub mod anime;

#[cfg(feature = "plugin")]
#[plugin(media_types = ["Anime"])]
mod anime_plugin {
    use cosmox_api::api::bindings::cosmox::plugin::cosmox_types;
    use cosmox_api::api::bindings::exports::cosmox::plugin::{
        command_handler, core_lifecycle, host_notifier, telemetry_reporter,
    };
    use cosmox_api::event::payloads::OnMetadataRawTreeReadyEventContext;
    use cosmox_api::handle::{MetadataView, PathMappingView};

    #[on_load]
    fn on_load(config: core_lifecycle::ConfigData) -> core_lifecycle::PluginResult {
        log::info!(
            "on loading anime base plugin: id={}, name={}",
            config.id,
            config.name
        );
        core_lifecycle::PluginResult::Ok
    }

    #[on_event(OnMetadataRawTreeReady({ r#type: vec!["Anime".into()] }))]
    fn handle_metadata_ready(
        _data: OnMetadataRawTreeReadyEventContext,
        metadata_view: MetadataView,
        _path_mapping_view: PathMappingView,
    ) -> host_notifier::PluginResult {
        log::info!("metadata raw tree ready, rebuilding tree structure");
        super::anime::action::rebuild_metadata_tree(metadata_view);
        host_notifier::PluginResult::Ok
    }

    #[health]
    fn get_health() -> telemetry_reporter::PluginHealthStatus {
        telemetry_reporter::PluginHealthStatus {
            status: cosmox_types::PluginStatus::Ok,
            message: None,
            metrics: None,
        }
    }

    #[on_command]
    fn execute_command(command_name: String, args: Vec<String>) -> command_handler::PluginResult {
        log::info!("execute command: {command_name} args: {args:?}");
        command_handler::PluginResult::Ok
    }
}
