use std::time::{
    Duration,
    Instant,
};

/// A message to be displayed to the user, one line max
pub struct Message {
    pub markdown: String,
    /// when the message was first displayed
    pub display_start: Option<Instant>,
    /// minimal duration to display the message
    pub display_duration: Duration,
}

impl Message {
    /// build a short message, typically to answer to a user action
    /// (thus when the user is looking at bacon)
    pub fn short(markdown: String) -> Self {
        Self {
            markdown,
            display_start: None,
            display_duration: Duration::from_secs(5),
        }
    }
}
