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
