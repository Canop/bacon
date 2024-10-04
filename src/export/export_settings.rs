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
            state.mission.workspace_root.join(&self.path)
        } else {
            self.path.to_path_buf()
        };
        info!("exporting to {:?}", path);
        match self.exporter {
            Exporter::Analysis => {
                let analysis_export =
                    AnalysisExport::build(state.mission.analyzer(), &state.cmd_result);
                let Some(analysis_export) = analysis_export else {
                    info!("No analysis to export");
                    return Ok(());
                };
                let json = serde_json::to_string_pretty(&analysis_export)?;
                std::fs::write(&path, json)?;
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
