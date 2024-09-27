use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Exporter {
    Analysis,
    JsonReport,
    Locations,
}
