use async_trait::async_trait;
use std::path::PathBuf;

use crate::add_torrent::{AddTorrentAction, AddTorrentOptions};
use crate::client::ClientAction;
use crate::get_torrents::{FilterOptions, Torrent};
use crate::{QBittorrentClientTrait, Response};
use rogue_logging::Failure;

/// Mock client for testing without live API calls
///
/// Set return values using the builder pattern, then use as `dyn QBittorrentClientTrait`.
#[derive(Clone, Debug)]
pub struct MockQBittorrentClient {
    get_torrents: Option<Response<Vec<Torrent>>>,
    add_torrents: Option<Response<bool>>,
}

impl MockQBittorrentClient {
    /// Create a new mock client with no configured return values
    #[must_use]
    pub fn new() -> Self {
        Self {
            get_torrents: None,
            add_torrents: None,
        }
    }

    /// Configure the return value for `get_torrents`
    #[must_use]
    pub fn with_get_torrents(mut self, response: Response<Vec<Torrent>>) -> Self {
        self.get_torrents = Some(response);
        self
    }

    /// Configure the return value for `add_torrent` and `add_torrents`
    #[must_use]
    pub fn with_add_torrents(mut self, response: Response<bool>) -> Self {
        self.add_torrents = Some(response);
        self
    }
}

impl Default for MockQBittorrentClient {
    fn default() -> Self {
        Self {
            get_torrents: Some(Response {
                status_code: Some(200),
                result: Some(vec![Torrent::mock()]),
            }),
            add_torrents: Some(Response {
                status_code: Some(200),
                result: Some(true),
            }),
        }
    }
}

#[async_trait]
impl QBittorrentClientTrait for MockQBittorrentClient {
    async fn get_torrents(
        &self,
        _filters: FilterOptions,
    ) -> Result<Response<Vec<Torrent>>, Failure<ClientAction>> {
        Ok(self
            .get_torrents
            .clone()
            .expect("MockQBittorrentClient: get_torrents not set"))
    }
    async fn add_torrent(
        &self,
        options: AddTorrentOptions,
        torrent: PathBuf,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>> {
        self.add_torrents(options, vec![torrent]).await
    }
    async fn add_torrents(
        &self,
        _options: AddTorrentOptions,
        _torrents: Vec<PathBuf>,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>> {
        Ok(self
            .add_torrents
            .clone()
            .expect("MockQBittorrentClient: add_torrents not set"))
    }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing, reason = "indexing after length validation")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_get_torrents_returns_configured_value() {
        let mock = MockQBittorrentClient::default();
        let filters = FilterOptions::default();
        let result = mock.get_torrents(filters).await;
        let response = result.expect("get_torrents should succeed");
        assert_eq!(response.status_code, Some(200));
        let torrents = response.result.expect("result should be present");
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].name, "Artist - Album [2023] [WEB FLAC]");
    }

    #[tokio::test]
    async fn mock_default_has_all_ok_responses() {
        let mock = MockQBittorrentClient::default();
        assert!(mock.get_torrents(FilterOptions::default()).await.is_ok());
        assert!(mock
            .add_torrents(AddTorrentOptions::default(), vec![])
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn mock_can_be_called_multiple_times() {
        let mock = MockQBittorrentClient::default();
        let filters = FilterOptions::default();
        let result1 = mock.get_torrents(filters).await;
        let filters = FilterOptions::default();
        let result2 = mock.get_torrents(filters).await;
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
