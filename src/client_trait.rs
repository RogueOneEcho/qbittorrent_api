use async_trait::async_trait;
use std::path::PathBuf;

use crate::add_torrent::{AddTorrentAction, AddTorrentOptions};
use crate::client::ClientAction;
use crate::get_torrents::{FilterOptions, Torrent};
use crate::Response;
use rogue_logging::Failure;

/// Trait for qBittorrent API operations
///
/// Implemented by [`QBittorrentClient`] for production use and
/// [`MockQBittorrentClient`] (with `mock` feature) for testing.
#[async_trait]
pub trait QBittorrentClientTrait: Send + Sync {
    /// Get all torrents matching the filter
    async fn get_torrents(
        &self,
        filters: FilterOptions,
    ) -> Result<Response<Vec<Torrent>>, Failure<ClientAction>>;

    /// Add torrent from file
    async fn add_torrent(
        &self,
        options: AddTorrentOptions,
        torrent: PathBuf,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>>;

    /// Add torrents from file
    async fn add_torrents(
        &self,
        options: AddTorrentOptions,
        torrents: Vec<PathBuf>,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>>;
}
