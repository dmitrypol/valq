/// Represents the type of queue, either the main, delayed or the dead-letter queue (DLQ).
#[derive(Debug, PartialEq)]
pub(crate) enum QType {
    /// Main queue type.
    Main,
    /// Dead-letter queue type.
    Dlq,
    /// Delayed queue type
    Delayed,
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
            "dlq" => Self::Dlq,
            "delayed" => Self::Delayed,
            _ => Self::Main, // Default value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(QType::from_str("dlq"), QType::Dlq);
        assert_eq!(QType::from_str("delayed"), QType::Delayed);
        assert_eq!(QType::from_str("main"), QType::Main);
        assert_eq!(QType::from_str(""), QType::Main); // Default case
    }
}
