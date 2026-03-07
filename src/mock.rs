use async_trait::async_trait;
use std::path::PathBuf;

use crate::add_torrent::{AddTorrentAction, AddTorrentOptions};
use crate::client::ClientAction;
use crate::get_torrents::{FilterOptions, Torrent};
use crate::{QBittorrentClientTrait, Response, Status};
use rogue_logging::Failure;

/// Mock client for testing without live API calls
///
/// Set return values using the builder pattern, then use as `dyn QBittorrentClientTrait`.
#[derive(Clone, Debug)]
#[allow(clippy::struct_field_names)]
pub struct MockQBittorrentClient {
    login_returns: Option<Status>,
    get_torrents_returns: Option<Response<Vec<Torrent>>>,
    add_torrents_returns: Option<Response<bool>>,
}

impl MockQBittorrentClient {
    /// Create a new mock client with no configured return values
    #[must_use]
    pub fn new() -> Self {
        Self {
            login_returns: None,
            get_torrents_returns: None,
            add_torrents_returns: None,
        }
    }

    /// Configure the return value for `login`
    #[must_use]
    pub fn with_login(mut self, status: Status) -> Self {
        self.login_returns = Some(status);
        self
    }

    /// Configure the return value for `get_torrents`
    #[must_use]
    pub fn with_get_torrents(mut self, response: Response<Vec<Torrent>>) -> Self {
        self.get_torrents_returns = Some(response);
        self
    }

    /// Configure the return value for `add_torrent` and `add_torrents`
    #[must_use]
    pub fn with_add_torrents(mut self, response: Response<bool>) -> Self {
        self.add_torrents_returns = Some(response);
        self
    }
}

impl Default for MockQBittorrentClient {
    fn default() -> Self {
        Self {
            login_returns: Some(Status::Success),
            get_torrents_returns: Some(Response {
                status_code: Some(200),
                result: Some(vec![Torrent::mock()]),
            }),
            add_torrents_returns: Some(Response {
                status_code: Some(200),
                result: Some(true),
            }),
        }
    }
}

#[async_trait]
impl QBittorrentClientTrait for MockQBittorrentClient {
    async fn login(&mut self) -> Result<Status, Failure<ClientAction>> {
        Ok(self
            .login_returns
            .clone()
            .expect("MockQBittorrentClient: login_returns not set"))
    }
    async fn get_torrents(
        &mut self,
        _filters: FilterOptions,
    ) -> Result<Response<Vec<Torrent>>, Failure<ClientAction>> {
        Ok(self
            .get_torrents_returns
            .clone()
            .expect("MockQBittorrentClient: get_torrents_returns not set"))
    }
    async fn add_torrent(
        &mut self,
        options: AddTorrentOptions,
        torrent: PathBuf,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>> {
        self.add_torrents(options, vec![torrent]).await
    }
    async fn add_torrents(
        &mut self,
        _options: AddTorrentOptions,
        _torrents: Vec<PathBuf>,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>> {
        Ok(self
            .add_torrents_returns
            .clone()
            .expect("MockQBittorrentClient: add_torrents_returns not set"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_login_returns_configured_value() {
        let mut mock = MockQBittorrentClient::new().with_login(Status::Success);
        let result = mock.login().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Status::Success);
    }

    #[tokio::test]
    async fn mock_get_torrents_returns_configured_value() {
        let mut mock = MockQBittorrentClient::default();
        let filters = FilterOptions::default();
        let result = mock.get_torrents(filters).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status_code, Some(200));
        let torrents = response.result.unwrap();
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].name, "Artist - Album [2023] [WEB FLAC]");
    }

    #[tokio::test]
    async fn mock_default_has_all_ok_responses() {
        let mut mock = MockQBittorrentClient::default();
        assert!(mock.login().await.is_ok());
        assert!(mock.get_torrents(FilterOptions::default()).await.is_ok());
        assert!(mock
            .add_torrents(AddTorrentOptions::default(), vec![])
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn mock_can_be_called_multiple_times() {
        let mut mock = MockQBittorrentClient::default();
        let filters = FilterOptions::default();
        let result1 = mock.get_torrents(filters).await;
        let filters = FilterOptions::default();
        let result2 = mock.get_torrents(filters).await;
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
