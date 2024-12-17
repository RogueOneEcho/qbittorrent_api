use crate::client::deserialize_response;
use crate::{QBittorrentClient, Response};
use reqwest::Method;
use rogue_logging::Error;

impl QBittorrentClient {
    /// Login and get a session cookie
    ///
    /// # See Also
    /// - <https://deluge.readthedocs.io/en/latest/devguide/how-to/curl-jsonrpc.html>
    pub async fn login(&mut self) -> Result<Response<bool>, Error> {
        let method = Method::GET;
        let endpoint = "login";
        let username = self.username.clone();
        let password = self.password.clone();
        let data = vec![
            ("username", username.as_str()),
            ("password", password.as_str()),
        ];
        let response = self.request(method.clone(), endpoint, &data).await?;
        deserialize_response(method, endpoint, response).await
    }
}

#[cfg(test)]
mod tests {
    use crate::QBittorrentClient;
    use crate::QBittorrentClientOptions;
    use log::trace;
    use reqwest::cookie::CookieStore;
    use reqwest::Url;
    use rogue_config::{OptionsProvider, YamlOptionsProvider};
    use rogue_logging::{Error, LoggerBuilder};

    #[tokio::test]
    async fn login() -> Result<(), Error> {
        // Arrange
        let _ = LoggerBuilder::new().create();
        let options: QBittorrentClientOptions = YamlOptionsProvider::get()?;
        let mut client = QBittorrentClient::from_options(options);

        // Act
        let response = client.login().await?;
        trace!("{}", response.to_json_pretty());

        // Assert
        let result = response.get_result("login")?;
        assert!(result);
        let url = Url::parse(&client.host.clone()).expect("url should parse");
        let cookies = client.cookies.cookies(&url);
        trace!("{cookies:?}");
        assert!(cookies.is_some());
        Ok(())
    }
}
