use {
    schemars::{
        JsonSchema,
        Schema,
        SchemaGenerator,
        json_schema,
    },
    serde::{
        Deserialize,
        Deserializer,
        de,
    },
    std::{
        borrow::Cow,
        fmt,
    },
    termimad::{
        crossterm::style::Color,
        parse_color,
    },
};

/// A color of the skin, read from configuration either as an 8-bit ANSI
/// code (eg `208`) or as a string in termimad's syntax (eg `"rgb(255, 187, 0)"`,
/// `"#fb0"`, `"gray(5)"`, `"ansi(208)"`, `"darkred"`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkinColor(Color);

impl SkinColor {
    pub const fn ansi(code: u8) -> Self {
        Self(Color::AnsiValue(code))
    }
    pub fn color(self) -> Color {
        self.0
    }
}

struct SkinColorVisitor;

impl de::Visitor<'_> for SkinColorVisitor {
    type Value = SkinColor;
    fn expecting(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.write_str(
            "an ANSI color code in [0, 255] or a color string like \"rgb(255, 187, 0)\", \"#fb0\", \"gray(5)\" or \"darkred\"",
        )
    }
    fn visit_i64<E: de::Error>(
        self,
        v: i64,
    ) -> Result<SkinColor, E> {
        u8::try_from(v)
            .map(SkinColor::ansi)
            .map_err(|_| E::invalid_value(de::Unexpected::Signed(v), &self))
    }
    fn visit_u64<E: de::Error>(
        self,
        v: u64,
    ) -> Result<SkinColor, E> {
        u8::try_from(v)
            .map(SkinColor::ansi)
            .map_err(|_| E::invalid_value(de::Unexpected::Unsigned(v), &self))
    }
    fn visit_str<E: de::Error>(
        self,
        s: &str,
    ) -> Result<SkinColor, E> {
        parse_color(s)
            .map(SkinColor)
            .map_err(|e| E::custom(format!("{e}: {s:?}")))
    }
}

impl<'de> Deserialize<'de> for SkinColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SkinColorVisitor)
    }
}

impl JsonSchema for SkinColor {
    fn schema_name() -> Cow<'static, str> {
        "SkinColor".into()
    }
    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::SkinColor").into()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "A color, either an 8-bit ANSI code or a string: \"rgb(255, 187, 0)\", \"#fb0\", \"gray(5)\", \"ansi(208)\", or a color name like \"darkred\".",
            "anyOf": [
                {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 255
                },
                {
                    "type": "string"
                }
            ]
        })
    }
    fn inline_schema() -> bool {
        false
    }
}

#[test]
fn test_skin_color_deserialization() {
    #[derive(Deserialize)]
    struct Conf {
        c: SkinColor,
    }
    let parse = |toml: &str| toml::from_str::<Conf>(toml).map(|conf| conf.c.color());
    assert_eq!(parse("c = 208").unwrap(), Color::AnsiValue(208));
    assert_eq!(parse(r#"c = "ansi(208)""#).unwrap(), Color::AnsiValue(208));
    assert_eq!(
        parse(r##"c = "#fb0""##).unwrap(),
        Color::Rgb {
            r: 255,
            g: 187,
            b: 0
        },
    );
    assert_eq!(
        parse(r#"c = "rgb(1, 2, 3)""#).unwrap(),
        Color::Rgb { r: 1, g: 2, b: 3 },
    );
    assert_eq!(
        parse(r##"c = "#FFBB00""##).unwrap(),
        Color::Rgb {
            r: 255,
            g: 187,
            b: 0
        },
    );
    assert_eq!(
        parse(r#"c = "RGB(1,2,3)""#).unwrap(),
        Color::Rgb { r: 1, g: 2, b: 3 },
    );
    assert_eq!(parse(r#"c = "gray(5)""#).unwrap(), Color::AnsiValue(237));
    assert_eq!(parse(r#"c = "grey(5)""#).unwrap(), Color::AnsiValue(237));
    assert_eq!(parse(r#"c = "Gray(23)""#).unwrap(), Color::AnsiValue(255));
    assert_eq!(parse(r#"c = "red""#).unwrap(), Color::Red);
    assert_eq!(parse(r#"c = "DarkRed""#).unwrap(), Color::DarkRed);
    assert_eq!(parse(r#"c = "DARKRED""#).unwrap(), Color::DarkRed);
    assert_eq!(parse(r#"c = "Magenta""#).unwrap(), Color::Magenta);
    assert_eq!(parse(r#"c = "darkblue""#).unwrap(), Color::DarkBlue);
    assert!(parse(r#"c = "gray(24)""#).is_err());
    assert!(parse("c = 256").is_err());
    assert!(parse("c = -1").is_err());
    let err = parse(r#"c = "nope""#).unwrap_err().to_string();
    assert!(err.contains("not a recognized color"), "{err}");
}
