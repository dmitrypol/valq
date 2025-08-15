pub(crate) enum QType {
    Main,
    Dlq,
}

impl QType {
    pub(crate) fn from_str(s: &str) -> Self {
        match s {
            "dlq" => QType::Dlq,
            _ => QType::Main, // Default value
        }
    }
}
