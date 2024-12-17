use crate::client::handle_status_response;
use crate::{QBittorrentClient, Status};
use reqwest::Method;
use rogue_logging::Error;

impl QBittorrentClient {
    /// Login and get a session cookie
    ///
    /// # See Also
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#login>
    pub async fn login(&mut self) -> Result<Status, Error> {
        let method = Method::POST;
        let endpoint = "/auth/login";
        let username = self.username.clone();
        let password = self.password.clone();
        let data = vec![
            ("username", username.as_str()),
            ("password", password.as_str()),
        ];
        let response = self.request(method.clone(), endpoint, &data).await?;
        handle_status_response(method, endpoint, response).await
    }
}

#[cfg(test)]
mod tests {
    use crate::QBittorrentClientOptions;
    use crate::{QBittorrentClient, Status};
    use log::trace;
    use reqwest::cookie::CookieStore;
    use reqwest::Url;
    use rogue_config::{OptionsProvider, YamlOptionsProvider};
    use rogue_logging::Verbosity::Trace;
    use rogue_logging::{Error, LoggerBuilder};

    #[tokio::test]
    async fn login() -> Result<(), Error> {
        // Arrange
        let _ = LoggerBuilder::new()
            .with_exclude_filter("reqwest".to_owned())
            .with_exclude_filter("cookie".to_owned())
            .with_verbosity(Trace)
            .create();
        let options: QBittorrentClientOptions = YamlOptionsProvider::get()?;
        let mut client = QBittorrentClient::from_options(options);

        // Act
        let status = client.login().await?;

        // Assert
        assert_eq!(status, Status::Success);
        let url = Url::parse(&client.host.clone()).expect("url should parse");
        let cookies = client.cookies.cookies(&url);
        trace!("{cookies:?}");
        assert!(cookies.is_some());
        Ok(())
    }
}
