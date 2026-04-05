//! Generic API response wrapper with status code validation.

use crate::client::ClientAction;
use reqwest::StatusCode;
use rogue_logging::Failure;
use serde::{Deserialize, Serialize};

/// API response containing a status code and deserialized result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response<T> {
    /// HTTP status code returned by the API.
    pub status_code: Option<u16>,
    /// Deserialized response body.
    pub result: Option<T>,
}

impl<T: Serialize> Response<T> {
    /// Validate the status code and extract the result.
    ///
    /// Returns an error if:
    /// - Status code is not set
    /// - Status code is not valid
    /// - Status code is not successful
    /// - Result is not set
    pub fn get_result(self, action: &str) -> Result<T, Failure<ClientAction>> {
        let status_code_num = self.status_code.ok_or_else(|| {
            Failure::from_action(ClientAction::ValidateResponse)
                .with("reason", "status code is not set")
                .with("action", action)
        })?;
        let status_code = StatusCode::from_u16(status_code_num).map_err(|_| {
            Failure::from_action(ClientAction::ValidateResponse)
                .with("reason", "status code is invalid")
                .with("status_code", status_code_num.to_string())
                .with("action", action)
        })?;
        if !status_code.is_success() {
            let status_message = match status_code.canonical_reason() {
                None => status_code_num.to_string(),
                Some(status_message) => status_message.to_owned(),
            };
            return Err(Failure::from_action(ClientAction::ValidateResponse)
                .with(
                    "reason",
                    format!("status code indicated failure: {status_message}"),
                )
                .with("status_code", status_code_num.to_string())
                .with("action", action));
        }
        if let Some(result) = self.result {
            Ok(result)
        } else {
            Err(Failure::from_action(ClientAction::ValidateResponse)
                .with("reason", "result is not set")
                .with("action", action))
        }
    }

    /// Serialize the response as pretty-printed JSON.
    #[cfg(test)]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_result_success() {
        let response: Response<bool> = Response {
            status_code: Some(200),
            result: Some(true),
        };
        let result = response.get_result("test");
        assert!(result.is_ok());
        assert!(result.expect("should be ok"));
    }

    #[test]
    fn get_result_missing_status_code() {
        let response: Response<bool> = Response {
            status_code: None,
            result: Some(true),
        };
        let result = response.get_result("test");
        assert!(result.is_err());
    }

    #[test]
    fn get_result_failure_status_code() {
        let response: Response<bool> = Response {
            status_code: Some(403),
            result: Some(false),
        };
        let result = response.get_result("test");
        assert!(result.is_err());
    }

    #[test]
    fn get_result_missing_result() {
        let response: Response<bool> = Response {
            status_code: Some(200),
            result: None,
        };
        let result = response.get_result("test");
        assert!(result.is_err());
    }
}
