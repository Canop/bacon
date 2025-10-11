use {
    crate::*,
    cargo_metadata::diagnostic::{
        Diagnostic,
        DiagnosticSpan,
    },
    serde::Serialize,
};

/// An export in progress for the `cargo_json` analyzer
pub struct CargoJsonExport {
    pub name: String,
    /// The written data to write to the file
    pub export: String,
    pub line_template: iq::Template,
}

/// The data provided to the template, once per span
#[derive(Debug, Clone, Serialize)]
struct OnSpanData<'d> {
    diagnostic: &'d Diagnostic,
    span: &'d DiagnosticSpan,
}

impl CargoJsonExport {
    pub fn new(
        name: String,
        settings: &ExportSettings,
    ) -> Self {
        Self {
            name,
            export: String::new(),
            line_template: iq::Template::new(&settings.line_format),
        }
    }
    pub fn receive_diagnostic(
        &mut self,
        diagnostic: &Diagnostic,
    ) {
        for span in &diagnostic.spans {
            let data = {
                // This is a diagnostic that originates from a proc-macro.
                if let Some(expansion) = &span.expansion {
                    OnSpanData {
                        diagnostic,
                        span: &expansion.span,
                    }
                } else {
                    OnSpanData { diagnostic, span }
                }
            };
            let line = self.line_template.render(&data);
            if !line.is_empty() {
                self.export.push_str(&line);
                self.export.push('\n');
            }
        }
    }
}
