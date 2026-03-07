use async_trait::async_trait;
use std::path::PathBuf;

use crate::add_torrent::{AddTorrentAction, AddTorrentOptions};
use crate::client::ClientAction;
use crate::get_torrents::{FilterOptions, Torrent};
use crate::{Response, Status};
use rogue_logging::Failure;

/// Trait for qBittorrent API operations
///
/// Implemented by [`QBittorrentClient`] for production use and
/// [`MockQBittorrentClient`] (with `mock` feature) for testing.
#[async_trait]
pub trait QBittorrentClientTrait: Send {
    /// Login and get a session cookie
    async fn login(&mut self) -> Result<Status, Failure<ClientAction>>;

    /// Get all torrents matching the filter
    async fn get_torrents(
        &mut self,
        filters: FilterOptions,
    ) -> Result<Response<Vec<Torrent>>, Failure<ClientAction>>;

    /// Add torrent from file
    async fn add_torrent(
        &mut self,
        options: AddTorrentOptions,
        torrent: PathBuf,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>>;

    /// Add torrents from file
    async fn add_torrents(
        &mut self,
        options: AddTorrentOptions,
        torrents: Vec<PathBuf>,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>>;
}
