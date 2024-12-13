
/// The kind of projec/context, as it impacts computing features,
/// files to watch, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextNature {
    Cargo,
    Other,
}
