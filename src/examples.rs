/// A bacon launch example to display in the --help message
pub struct Example {
    pub title: &'static str,
    pub cmd: &'static str,
}

pub static EXAMPLES_TEMPLATE: &str = "
**Examples:**

${examples
*${example-number})* ${example-title}: `${example-cmd}`
}
";

pub static EXAMPLES: &[Example] = &[
    Example {
        title: "Start with the default job",
        cmd: "bacon",
    },
    Example {
        title: "Start with a specific job",
        cmd: "bacon clippy",
    },
    Example {
        title: "Start with features",
        cmd: "bacon --features clipboard",
    },
    Example {
        title: "Start a specific job on another path",
        cmd: "bacon ../broot test",
    },
    Example {
        title: "Start in summary mode",
        cmd: "bacon -s",
    },
];

