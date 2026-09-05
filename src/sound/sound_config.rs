use {
    crate::*,
    schemars::JsonSchema,
    serde::Deserialize,
    std::collections::HashMap,
};

/// Sound configuration.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, JsonSchema)]
pub struct SoundConfig {
    /// Whether sound notifications should be played.
    pub enabled: Option<bool>,

    /// Base volume, acting as a multiplier for the volume of specific sounds.
    pub base_volume: Option<Volume>,
    pub collection: Option<HashMap<String, String>>,
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
        #[cfg(feature = "sound")]
        // Load sounds configured in `[sound.collection]`.
        // This doesn't "apply" this item in the sense other items are applied;
        // but rather, add them to `super::play_sound::SOUNDS` so they can be
        // looked up later.
        // Ideally, they should be loaded at some more suitable point, or even
        // passed to `super::play_sound::play_sound()` to achieve some degree
        // of on-demand loading. But current structure of `SoundPlayer` makes
        // that difficult.
        if let Some(ref collection) = sc.collection {
            let mut self_collection = self.collection.get_or_insert_default();
            for (name, path) in collection {
                if !self_collection.contains_key(name) {
                    if let Err(e) = crate::sound::play_sound::add_sound(name, path) {
                        warn!("failed to add sound(name = {name}, path = {path}): {e:?}");
                    }
                    self_collection.insert(name.clone(), path.clone());
                }
            }
        }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
    }
    pub fn get_base_volume(&self) -> Volume {
        self.base_volume.unwrap_or_default()
    }
}
