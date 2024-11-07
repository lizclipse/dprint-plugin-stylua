use dprint_core::{
    configuration::{self, ConfigKeyMap, GlobalConfiguration, RECOMMENDED_GLOBAL_CONFIGURATION},
    generate_plugin_code,
    plugins::{
        FileMatchingInfo, FormatResult, PluginInfo, PluginResolveConfigurationResult,
        SyncFormatRequest, SyncHostFormatRequest, SyncPluginHandler,
    },
};
use stylua_lib::{LineEndings, OutputVerification};

use crate::config::Configuration;

pub struct StyluaPluginHandler;

impl SyncPluginHandler<Configuration> for StyluaPluginHandler {
    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<Configuration> {
        let mut config = config;
        let mut diagnostics = vec![];

        let default_config = stylua_lib::Config::default();

        let resolved_config = Configuration {
            line_width: configuration::get_value(
                &mut config,
                "lineWidth",
                global_config
                    .line_width
                    .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.line_width),
                &mut diagnostics,
            ),
            use_tabs: configuration::get_value(
                &mut config,
                "useTabs",
                global_config
                    .use_tabs
                    .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.use_tabs),
                &mut diagnostics,
            ),
            indent_width: configuration::get_value(
                &mut config,
                "indentWidth",
                global_config
                    .indent_width
                    .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.indent_width),
                &mut diagnostics,
            ),
            new_line_kind: configuration::get_value(
                &mut config,
                "newLineKind",
                global_config
                    .new_line_kind
                    .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.new_line_kind),
                &mut diagnostics,
            ),
            verify: configuration::get_value(&mut config, "verify", false, &mut diagnostics),
            quote_style: configuration::get_value(
                &mut config,
                "quoteStyle",
                default_config.quote_style,
                &mut diagnostics,
            ),
            call_parentheses: configuration::get_value(
                &mut config,
                "callParentheses",
                default_config.call_parentheses,
                &mut diagnostics,
            ),
            collapse_simple_statement: configuration::get_value(
                &mut config,
                "collapseSimpleStatement",
                default_config.collapse_simple_statement,
                &mut diagnostics,
            ),
            sort_requires: configuration::get_value(
                &mut config,
                "sortRequires",
                default_config.sort_requires.enabled,
                &mut diagnostics,
            ),
        };

        diagnostics.extend(configuration::get_unknown_property_diagnostics(config));

        PluginResolveConfigurationResult {
            diagnostics,
            config: resolved_config,
            file_matching: FileMatchingInfo {
                file_extensions: vec!["lua".to_owned()],
                file_names: vec![],
            },
        }
    }

    fn plugin_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "stylua".to_string(),
            help_url: concat!(env!("CARGO_PKG_REPOSITORY"), "#readme").to_string(),
            config_schema_url: "".to_string(),
            update_url: Some("https://plugins.dprint.dev/RubixDev/stylua/latest.json".to_string()),
        }
    }

    fn license_text(&mut self) -> String {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/LICENSE")).into()
    }

    fn format(
        &mut self,
        SyncFormatRequest {
            file_bytes, config, ..
        }: SyncFormatRequest<Configuration>,
        _format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
    ) -> FormatResult {
        let file_text = String::from_utf8(file_bytes)?;

        let mut stylua_config = stylua_lib::Config::from(config);
        stylua_config.line_endings =
            match configuration::resolve_new_line_kind(&file_text, config.new_line_kind) {
                "\r\n" => LineEndings::Windows,
                "\n" => LineEndings::Unix,
                // Fall back to \n in case upstream function changes
                _ => LineEndings::Unix,
            };

        let result = stylua_lib::format_code(
            &file_text,
            stylua_config,
            None,
            match config.verify {
                true => OutputVerification::Full,
                false => OutputVerification::None,
            },
        )?;
        if result == file_text {
            Ok(None)
        } else {
            Ok(Some(result.into_bytes()))
        }
    }

    fn check_config_updates(
        &self,
        _message: dprint_core::plugins::CheckConfigUpdatesMessage,
    ) -> anyhow::Result<Vec<dprint_core::plugins::ConfigChange>> {
        todo!()
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
generate_plugin_code!(StyluaPluginHandler, StyluaPluginHandler);
