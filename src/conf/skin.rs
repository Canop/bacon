use {
    paste::paste,
    serde::Deserialize,
    termimad::crossterm::style::Color,
};

/// Define a `BaconSkin` struct with fields being u8 with default values.
macro_rules! BaconSkin {
    (
        $( $name:ident: $default:literal, )*
    ) => {
        paste! {
            $(
                #[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
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
            #[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
            pub struct BaconSkin {
                $(
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
    status_fg: 252, // foreground color of the status line
    status_bg: 239, // background color of the status line
    key_fg: 204, // generally used for key shortcuts
    status_key_fg: 204, // key shortcuts when displayed in the status line
    project_name_badge_fg: 255, // foreground color of the project name badge
    project_name_badge_bg: 240, // background color of the project name badge
    job_label_badge_fg: 235, // foreground color of the job label badge
    job_label_badge_bg: 204, // background color of the job label badge
    errors_badge_fg: 235, // foreground color of the errors badge
    errors_badge_bg: 9, // background color of the errors badge
    test_fails_badge_fg: 235, // foreground color of the test fails badge
    test_fails_badge_bg: 208, // background color of the test fails badge
    test_pass_badge_fg: 254, // foreground color of the pass! badge
    test_pass_badge_bg: 2, // background color of the pass! badge
    warnings_badge_fg: 235, // foreground color of the warnings badge
    warnings_badge_bg: 11, // background color of the warnings badge
    command_error_badge_fg: 235, // foreground color of the command error badge
    command_error_badge_bg: 9, // background color of the command error badge
    dismissed_badge_fg: 235, // foreground color of the dismissed badge
    dismissed_badge_bg: 6, // background color of the dismissed badge
    change_badge_fg: 235, // foreground color of the change badge
    change_badge_bg: 6, // background color of the change badge
    computing_fg: 235, // foreground color of the "computing..." stripe
    computing_bg: 204, // background color of the "computing..." stripe
    found_fg: 208, // foreground color of search matches
    found_selected_bg: 208, // background color of a selected search match
    search_input_prefix_fg: 208, // foreground color of the '/' search prefix
    search_summary_fg: 208, // foreground color of the search summary
    menu_border: 234,
    menu_bg: 235,
    menu_item_bg: 235,
    menu_item_selected_bg: 239,
    menu_item_fg: 250,
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
