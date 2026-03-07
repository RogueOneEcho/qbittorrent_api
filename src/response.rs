use crate::client::ClientAction;
use reqwest::StatusCode;
use rogue_logging::Failure;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub status_code: Option<u16>,
    pub result: Option<T>,
}

impl<T: Serialize> Response<T> {
    /// Get the result
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

    #[cfg(test)]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| e.to_string())
    }
}
