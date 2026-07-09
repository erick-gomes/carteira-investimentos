#[derive(Debug, PartialEq, Eq)]
pub enum PostgresError {
    UniqueViolation,
    Unknown,
}

impl From<&str> for PostgresError {
    fn from(code: &str) -> Self {
        match code {
            "23505" => PostgresError::UniqueViolation,
            _ => PostgresError::Unknown,
        }
    }
}
