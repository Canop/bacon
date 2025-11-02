use {
    crate::*,
    schemars::JsonSchema,
    serde::Deserialize,
};

/// Configuration for mission sound notifications.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, JsonSchema)]
pub struct SoundConfig {
    /// Whether sound notifications should be played.
    pub enabled: Option<bool>,

    /// Base volume used when playing configured sounds.
    pub base_volume: Option<Volume>,
}

impl SoundConfig {
    pub fn apply(
        &mut self,
        sc: &SoundConfig,
    ) {
        if let Some(b) = sc.enabled {
            self.enabled = Some(b);
        }
        if let Some(bv) = sc.base_volume {
            self.base_volume = Some(bv);
        }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
    }
    pub fn get_base_volume(&self) -> Volume {
        self.base_volume.unwrap_or_default()
    }
}
