use {
    crate::*,
    std::{
        collections::HashMap,
        path::PathBuf,
    },
};

/// Settings for all exports
#[derive(Debug, Clone, Default)]
pub struct ExportsSettings {
    pub exports: HashMap<String, ExportSettings>,
}

impl ExportsSettings {
    pub fn set_locations_export_auto(
        &mut self,
        enabled: bool,
    ) {
        let locations_export = self
            .exports
            .entry("locations".to_string())
            .or_insert_with(default_locations_export_settings);
        locations_export.auto = enabled;
    }

    pub fn do_auto_exports(
        &self,
        state: &AppState<'_>,
    ) {
        for (name, export) in &self.exports {
            if export.auto {
                info!("doing auto export {:?}", name);
                if let Err(e) = export.do_export(state) {
                    error!("error while exporting {:?}: {:?}", name, e);
                }
            }
        }
    }

    pub fn do_named_export(
        &self,
        requested_name: &str,
        state: &AppState<'_>,
    ) {
        if let Some(export) = self.exports.get(requested_name) {
            if let Err(e) = export.do_export(state) {
                error!("error while exporting {:?}: {:?}", requested_name, e);
            }
        } else {
            warn!("no export named {:?}", requested_name);
        }
    }

    /// We apply different parts of the config, matching
    /// different generations of the config format:
    ///
    /// - the exports map (current)
    /// - the export object (recently deprecated since 2.22.0)
    /// - the export_locations field (deprecated since 2.9.0)
    ///
    /// FIXME Should we prevent having two exporters with the
    /// same path ?
    pub fn apply_config(
        &mut self,
        config: &Config,
    ) {
        // normal [exports] map
        for (name, ec) in &config.exports {
            if let Some(e) = self.exports.get_mut(name) {
                if let Some(exporter) = ec.exporter {
                    e.exporter = exporter;
                }
                if let Some(b) = ec.auto {
                    e.auto = b;
                }
                if let Some(p) = &ec.path {
                    e.path = p.clone();
                }
                if let Some(lf) = &ec.line_format {
                    e.line_format = lf.clone();
                }
                continue;
            }
            let exporter = match ec.exporter {
                Some(e) => e,
                None => match name.as_str() {
                    "analysis" => Exporter::Analysis,
                    "json-report" => Exporter::JsonReport,
                    "locations" => Exporter::Locations,
                    _ => {
                        warn!(
                            "Exporter not specified for export {:?}, using 'locations'",
                            name
                        );
                        Exporter::Locations
                    }
                },
            };
            let auto = ec.auto.unwrap_or(true);
            let path = ec.path.clone().unwrap_or_else(|| match exporter {
                Exporter::Analysis => default_analysis_path(),
                Exporter::Locations => default_locations_path(),
                Exporter::JsonReport => default_json_report_path(),
            });
            let line_format = ec.line_format.clone().unwrap_or_else(|| match exporter {
                Exporter::Locations => default_locations_line_format().to_string(),
                _ => "".to_string(),
            });
            self.exports.insert(name.clone(), ExportSettings {
                exporter,
                auto,
                path,
                line_format,
            });
        }

        // [export] object
        #[allow(deprecated)] // for compatibility
        if let Some(ec) = &config.export {
            match ec.exporter {
                Some(Exporter::Analysis) => {
                    let analysis_export = self
                        .exports
                        .entry("analysis".to_string())
                        .or_insert_with(default_analysis_export_settings);
                    if let Some(b) = ec.auto {
                        analysis_export.auto = b;
                    }
                    if let Some(p) = &ec.path {
                        analysis_export.path = p.clone();
                    }
                }
                Some(Exporter::JsonReport) => {
                    let json_report_export = self
                        .exports
                        .entry("json-report".to_string())
                        .or_insert_with(default_json_report_export_settings);
                    if let Some(b) = ec.auto {
                        json_report_export.auto = b;
                    }
                    if let Some(p) = &ec.path {
                        json_report_export.path = p.clone();
                    }
                }
                _ => {
                    let locations_export = self
                        .exports
                        .entry("locations".to_string())
                        .or_insert_with(default_locations_export_settings);
                    if let Some(b) = ec.auto {
                        locations_export.auto = b;
                    }
                    if let Some(p) = &ec.path {
                        locations_export.path = p.clone();
                    }
                    if let Some(lf) = &ec.line_format {
                        locations_export.line_format = lf.clone();
                    }
                }
            }
        }

        #[allow(deprecated)] // for compatibility
        if let Some(b) = config.export_locations {
            let locations_export = self
                .exports
                .entry("locations".to_string())
                .or_insert_with(default_locations_export_settings);
            locations_export.auto = b;
        }
    }
}

fn default_analysis_export_settings() -> ExportSettings {
    ExportSettings {
        exporter: Exporter::Analysis,
        auto: true,
        path: default_analysis_path(),
        line_format: "".to_string(), // not used
    }
}
fn default_json_report_export_settings() -> ExportSettings {
    ExportSettings {
        exporter: Exporter::JsonReport,
        auto: true,
        path: default_json_report_path(),
        line_format: "".to_string(), // not used
    }
}
fn default_locations_export_settings() -> ExportSettings {
    ExportSettings {
        exporter: Exporter::Locations,
        auto: true,
        path: default_locations_path(),
        line_format: default_locations_line_format().to_string(),
    }
}

pub fn default_locations_line_format() -> &'static str {
    "{kind} {path}:{line}:{column} {message}"
}

pub fn default_analysis_path() -> PathBuf {
    PathBuf::from("bacon-analysis.json")
}
pub fn default_json_report_path() -> PathBuf {
    PathBuf::from("bacon-report.json")
}
pub fn default_locations_path() -> PathBuf {
    PathBuf::from(".bacon-locations")
}
