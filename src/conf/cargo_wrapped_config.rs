use {
    super::Config,
    anyhow::*,
    serde::Deserialize,
    std::path::Path,
};

#[derive(Deserialize)]
struct CargoWrappedConfig {
    workspace: Option<WrappedMetadata>,
    package: Option<WrappedMetadata>,
}
#[derive(Deserialize)]
struct WrappedMetadata {
    metadata: Option<WrappedConfig>,
}
#[derive(Deserialize)]
struct WrappedConfig {
    bacon: Option<Config>,
}

pub fn load_config_from_cargo_toml(cargo_file_path: &Path) -> Result<Vec<Config>> {
    if !cargo_file_path.exists() {
        return Ok(Vec::default());
    }
    let cargo_toml = std::fs::read_to_string(cargo_file_path)?;
    let mut cargo: CargoWrappedConfig = toml::from_str(&cargo_toml)?;
    let mut configs = Vec::new();
    let worskpace_config = cargo
        .workspace
        .take()
        .and_then(|workspace| workspace.metadata)
        .and_then(|metadata| metadata.bacon);
    if let Some(config) = worskpace_config {
        configs.push(config);
    }
    let worskpace_config = cargo
        .package
        .take()
        .and_then(|package| package.metadata)
        .and_then(|metadata| metadata.bacon);
    if let Some(config) = worskpace_config {
        configs.push(config);
    }
    Ok(configs)
}
