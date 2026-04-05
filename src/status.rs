//! Login response status parsing.

use crate::status::Status::{Failure, Other, Success};
use serde::{Deserialize, Serialize};

/// Parsed status from a qBittorrent API text response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Status {
    /// API returned `"Ok."`
    Success,
    /// API returned `"Fails."`
    Failure,
    /// API returned an unrecognized response.
    Other(String),
}

impl From<&str> for Status {
    fn from(value: &str) -> Self {
        match value {
            "Ok." => Success,
            "Fails." => Failure,
            _ => Other(value.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_from_ok() {
        assert_eq!(Status::from("Ok."), Success);
    }

    #[test]
    fn status_from_fails() {
        assert_eq!(Status::from("Fails."), Failure);
    }

    #[test]
    fn status_from_other() {
        assert_eq!(
            Status::from("something else"),
            Other("something else".to_owned())
        );
    }
}
