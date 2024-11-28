use {
    crate::*,
    std::{
        fs::File,
        path::PathBuf,
    },
};

/// Settings for one export
#[derive(Debug, Clone)]
pub struct ExportSettings {
    pub exporter: Exporter,
    pub auto: bool,
    pub path: PathBuf,
    pub line_format: String,
}
impl ExportSettings {
    pub fn do_export(
        &self,
        state: &AppState<'_>,
    ) -> anyhow::Result<()> {
        let path = if self.path.is_relative() {
            state.mission.package_directory.join(&self.path)
        } else {
            self.path.to_path_buf()
        };
        info!("exporting to {:?}", path);
        match self.exporter {
            Exporter::Analysis => {
                error!("Aanlysis export not currently implemented");
            }
            Exporter::JsonReport => {
                let Some(report) = state.cmd_result.report() else {
                    info!("No report to export");
                    return Ok(());
                };
                let json = serde_json::to_string_pretty(&report)?;
                std::fs::write(&path, json)?;
            }
            Exporter::Locations => {
                let Some(report) = state.cmd_result.report() else {
                    info!("No report to export");
                    return Ok(());
                };
                let mut file = File::create(path)?;
                report.write_locations(&mut file, &state.mission, &self.line_format)?;
            }
        }
        Ok(())
    }
}
