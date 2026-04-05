//! Session authentication and cookie management.

use crate::client::{handle_status_response, ClientAction};
use crate::{QBittorrentClient, Status};
use reqwest::cookie::CookieStore;
use reqwest::Method;
use rogue_logging::Failure;

impl QBittorrentClient {
    /// Login and get a session cookie
    ///
    /// # See Also
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#login>
    pub(crate) async fn login(&self) -> Result<Status, Failure<ClientAction>> {
        let method = Method::POST;
        let endpoint = "/auth/login";
        let username = self.username.clone();
        let password = self.password.clone();
        let data = vec![
            ("username", username.as_str()),
            ("password", password.as_str()),
        ];
        let response = self.request(method.clone(), endpoint, &data).await?;
        handle_status_response(&method, endpoint, response).await
    }

    /// Check if the cookie jar contains a session cookie for the host
    pub(crate) fn has_session_cookie(&self) -> bool {
        let url = reqwest::Url::parse(&self.host).expect("host should be a valid URL");
        self.cookies.cookies(&url).is_some()
    }

    /// Login if no session cookie exists
    pub(crate) async fn ensure_login(&self) -> Result<(), Failure<ClientAction>> {
        if !self.has_session_cookie() {
            let status = self.login().await?;
            if status != Status::Success {
                return Err(
                    Failure::from_action(ClientAction::Login).with("status", format!("{status:?}"))
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::init_logger;
    use crate::QBittorrentClientOptions;
    use crate::{QBittorrentClient, Status};
    use rogue_config::{OptionsProvider, YamlOptionsProvider};
    use std::error::Error;

    #[tokio::test]
    #[ignore = "integration test requiring API credentials"]
    async fn login() -> Result<(), Box<dyn Error>> {
        // Arrange
        init_logger();
        let options: QBittorrentClientOptions =
            YamlOptionsProvider::get().map_err(|e| e.to_string())?;
        let client = QBittorrentClient::from_options(options);

        // Act
        let status = client.login().await?;

        // Assert
        assert_eq!(status, Status::Success);
        assert!(client.has_session_cookie());
        Ok(())
    }
}
