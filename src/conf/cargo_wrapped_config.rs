use {
    super::Config,
    anyhow::*,
    serde::Deserialize,
    std::path::Path,
};

#[derive(Deserialize)]
struct CargoWrappedConfig {
    package: Option<CargoPackageWrappedConfig>,
}
#[derive(Deserialize)]
struct CargoPackageWrappedConfig {
    metadata: Option<CargoPackageMetadataWrappedConfig>,
}
#[derive(Deserialize)]
struct CargoPackageMetadataWrappedConfig {
    bacon: Option<Config>,
}

pub fn load_config_from_cargo_toml(
    cargo_file_path: &Path,
) -> Result<Option<Config>> {
    if !cargo_file_path.exists() {
        return Ok(None);
    }
    let cargo_toml = std::fs::read_to_string(cargo_file_path)?;
    let cargo: CargoWrappedConfig = toml::from_str(&cargo_toml)?;
    let config = cargo.package
        .and_then(|package| package.metadata)
        .and_then(|metadata| metadata.bacon);
    Ok(config)
}
