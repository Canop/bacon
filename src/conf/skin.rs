use {
    paste::paste,
    schemars::JsonSchema,
    serde::Deserialize,
    termimad::crossterm::style::Color,
};

/// Define a `BaconSkin` struct with fields being u8 with default values.
macro_rules! BaconSkin {
    (
        $( $(#[$meta:meta])* $name:ident: $default:literal, )*
    ) => {
        paste! {
            $(
                $(#[$meta])*
                #[doc=concat!(" - default value: ", stringify!($default))]
                #[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, JsonSchema)]
                #[serde(untagged)]
                pub enum [<Defaulting$name:camel>] {
                    Set(u8),
                    #[default]
                    Unset,
                }
                impl [<Defaulting$name:camel>] {
                    pub fn value(self) -> u8 {
                        match self {
                            Self::Set(value) => value,
                            Self::Unset => $default,
                        }
                    }
                    pub fn apply(&mut self, other: Self) {
                        if let Self::Set(value) = other {
                            *self = Self::Set(value);
                        }
                    }
                    pub fn color(self) -> Color {
                        Color::AnsiValue(self.value())
                    }
                }
            )*
            /// Collection of optional color overrides for the Bacon UI.
            #[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, JsonSchema)]
            pub struct BaconSkin {
                $(
                    $(#[$meta])*
                    #[serde(default)]
                    pub $name: [<Defaulting$name:camel>],
                )*
            }
            impl BaconSkin {
                pub fn apply(&mut self, other: Self) {
                    $(
                        self.$name.apply(other.$name);
                    )*
                }
                $(
                    #[inline]
                    pub fn [<$name>](&self) -> u8 {
                        self.$name.value()
                    }
                )*
            }
        }
    }
}

// The colors of Bacon, with default values (ANSI color codes, in 0-255)
BaconSkin! {
    /// Foreground color of the status line
    status_fg: 252,
    /// Background color of the status line
    status_bg: 239,
    /// Foreground color used for key shortcuts in the UI
    key_fg: 204,
    /// Foreground color for key shortcuts displayed in the status line
    status_key_fg: 204,
    /// Foreground color of the project name badge
    project_name_badge_fg: 255,
    /// Background color of the project name badge
    project_name_badge_bg: 240,
    /// Foreground color of the job label badge
    job_label_badge_fg: 235,
    /// Background color of the job label badge
    job_label_badge_bg: 204,
    /// Foreground color of the errors badge
    errors_badge_fg: 235,
    /// Background color of the errors badge
    errors_badge_bg: 9,
    /// Foreground color of the failing-tests badge
    test_fails_badge_fg: 235,
    /// Background color of the failing-tests badge
    test_fails_badge_bg: 208,
    /// Foreground color of the passing-tests badge
    test_pass_badge_fg: 254,
    /// Background color of the passing-tests badge
    test_pass_badge_bg: 2,
    /// Foreground color of the warnings badge
    warnings_badge_fg: 235,
    /// Background color of the warnings badge
    warnings_badge_bg: 11,
    /// Foreground color of the command-error badge
    command_error_badge_fg: 235,
    /// Background color of the command-error badge
    command_error_badge_bg: 9,
    /// Foreground color of the dismissed badge
    dismissed_badge_fg: 235,
    /// Background color of the dismissed badge
    dismissed_badge_bg: 6,
    /// Foreground color of the change badge
    change_badge_fg: 235,
    /// Background color of the change badge
    change_badge_bg: 6,
    /// Foreground color of the "computing..." indicator
    computing_fg: 235,
    /// Background color of the "computing..." indicator
    computing_bg: 204,
    /// Foreground color of search matches
    found_fg: 208,
    /// Background color of the selected search match
    found_selected_bg: 208,
    /// Foreground color of the '/' search prefix
    search_input_prefix_fg: 208,
    /// Foreground color of the search summary
    search_summary_fg: 208,
    /// Border color used for menus
    menu_border: 234,
    /// Background color used for menus
    menu_bg: 235,
    /// Background color of individual menu items
    menu_item_bg: 235,
    /// Background color of the selected menu item
    menu_item_selected_bg: 239,
    /// Foreground color of menu items
    menu_item_fg: 250,
    /// Foreground color of the selected menu item
    menu_item_selected_fg: 255,
}

#[test]
fn test_bacon_skin_defaults() {
    let a = r"
        status_fg = 255
        status_key_fg = 200
        project_name_badge_fg = 0
    ";
    let mut a = toml::from_str::<BaconSkin>(a).unwrap();
    assert_eq!(a.status_fg(), 255);
    assert_eq!(a.status_bg(), 239);
    let b = r"
        status_key_fg = 206
        status_bg = 100
    ";
    let b = toml::from_str::<BaconSkin>(b).unwrap();
    a.apply(b);
    assert_eq!(a.status_fg(), 255);
    assert_eq!(a.status_bg(), 100);
    assert_eq!(a.key_fg(), 204);
    assert_eq!(a.status_key_fg(), 206);
    assert_eq!(a.project_name_badge_fg(), 0);
    assert_eq!(a.project_name_badge_bg(), 240);
}
