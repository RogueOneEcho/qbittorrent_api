use crate::status::Status::{Failure, Other, Success};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Success,
    Failure,
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
