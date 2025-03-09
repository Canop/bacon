use {
    crate::*,
    std::collections::HashMap,
    serde::Deserialize,
};

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct SoundConfig {
    pub enabled: Option<bool>,
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
            for (name, path) in collection {
                // Silently ignore failures adding sounds.
                // We might want to give the user some hints later.
                crate::sound::play_sound::add_sound(name, path).ok();
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
