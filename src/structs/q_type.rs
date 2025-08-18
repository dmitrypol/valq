/// Represents the type of a queue, either the main queue or the dead-letter queue (DLQ).
pub(crate) enum QType {
    /// Main queue type.
    Main,
    /// Dead-letter queue type.
    Dlq,
}

impl QType {
    /// Converts a string representation to a `QType` enum.
    ///
    /// # Arguments
    /// * `input` - A string slice representing the queue type.
    ///         Accepts "dlq" for the dead-letter queue; any other value defaults to the main queue.
    ///
    /// # Returns
    /// * `QType::Dlq` - If the input string is "dlq".
    /// * `QType::Main` - For any other input, including empty strings.
    pub(crate) fn from_str(input: &str) -> Self {
        match input {
            "dlq" => QType::Dlq,
            _ => QType::Main, // Default value
        }
    }
}
