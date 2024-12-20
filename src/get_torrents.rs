use crate::client::deserialize_response;
use crate::QBittorrentClient;
use crate::Response;
use reqwest::Method;
use rogue_logging::Error;
use serde::{Deserialize, Serialize};

impl QBittorrentClient {
    /// Get all torrents matching the filter
    ///
    /// # See Also
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#get-torrent-list>
    pub async fn get_torrents(
        &mut self,
        filters: FilterOptions,
    ) -> Result<Response<Vec<Torrent>>, Error> {
        let method = Method::GET;
        let endpoint = "/torrents/info";
        let response = self.request(method.clone(), endpoint, &filters).await?;
        let response = deserialize_response::<Vec<Torrent>>(method, endpoint, response).await?;
        Ok(Response {
            status_code: response.status_code,
            result: response.result,
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[allow(clippy::doc_markdown)]
pub struct FilterOptions {
    /// Filter torrent list by state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterState>,

    /// Get torrents with the given category.
    /// - Empty string means "without category".
    /// - Omitting this field means "any category".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Get torrents with the given tag.
    /// - Empty string means "without tag".
    /// - Omitting this field means "any tag".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Sort torrents by the given key.
    /// Any JSON field name from the torrent response can be used as the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,

    /// Enable reverse sorting. Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,

    /// Limit the number of torrents returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Set offset for pagination.
    /// If less than `0`, offset starts from the end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,

    /// Filter by torrent hashes.
    /// Multiple hashes can be separated by `|`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<String>,
}

/// Represents the allowed torrent states for filtering.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterState {
    All,
    Downloading,
    Seeding,
    Completed,
    Paused,
    Active,
    Inactive,
    Resumed,
    Stalled,
    StalledUploading,
    StalledDownloading,
    Errored,
}

/// Represents detailed information about a torrent.
#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Torrent {
    /// Time (Unix Epoch) when the torrent was added to the client.
    pub added_on: i64,

    /// Amount of data left to download (bytes).
    pub amount_left: u64,

    /// Whether this torrent is managed by Automatic Torrent Management.
    pub auto_tmm: bool,

    /// Percentage of file pieces currently available.
    pub availability: f64,

    /// Category of the torrent.
    pub category: String,

    /// Amount of transfer data completed (bytes).
    pub completed: u64,

    /// Time (Unix Epoch) when the torrent completed.
    pub completion_on: i64,

    /// Absolute path of torrent content.
    /// Root path for multifile torrents, absolute file path for singlefile torrents.
    pub content_path: String,

    /// Torrent download speed limit (bytes/s). `-1` if unlimited.
    pub dl_limit: i64,

    /// Torrent download speed (bytes/s).
    pub dlspeed: u64,

    /// Total amount of data downloaded.
    pub downloaded: u64,

    /// Amount of data downloaded in the current session.
    pub downloaded_session: u64,

    /// Torrent ETA (seconds).
    pub eta: i64,

    /// True if first and last pieces are prioritized.
    pub f_l_piece_prio: bool,

    /// True if force start is enabled for this torrent.
    pub force_start: bool,

    /// Torrent hash.
    pub hash: String,

    /// True if torrent is from a private tracker (added in 5.0.0).
    pub is_private: Option<bool>,

    /// Last time (Unix Epoch) when a chunk was downloaded/uploaded.
    pub last_activity: i64,

    /// Magnet URI corresponding to this torrent.
    pub magnet_uri: String,

    /// Maximum share ratio until the torrent is stopped from seeding/uploading.
    pub max_ratio: f64,

    /// Maximum seeding time (seconds) until torrent is stopped from seeding.
    pub max_seeding_time: i64,

    /// Torrent name.
    pub name: String,

    /// Number of seeds in the swarm.
    pub num_complete: u64,

    /// Number of leechers in the swarm.
    pub num_incomplete: u64,

    /// Number of leechers connected to.
    pub num_leechs: u64,

    /// Number of seeds connected to.
    pub num_seeds: u64,

    /// Torrent priority. Returns `-1` if queuing is disabled or torrent is in seed mode.
    pub priority: i64,

    /// Torrent progress (percentage/100).
    pub progress: f64,

    /// Torrent share ratio. Max ratio value: `9999`.
    pub ratio: f64,

    /// TODO: Difference from `max_ratio`?
    pub ratio_limit: f64,

    /// Path where this torrent's data is stored.
    pub save_path: String,

    /// Torrent elapsed time while complete (seconds).
    pub seeding_time: i64,

    /// TODO: Difference from `max_seeding_time`?
    pub seeding_time_limit: i64,

    /// Time (Unix Epoch) when this torrent was last seen complete.
    pub seen_complete: i64,

    /// True if sequential download is enabled.
    pub seq_dl: bool,

    /// Total size (bytes) of files selected for download.
    pub size: u64,

    /// Torrent state (e.g., "downloading", "seeding", "paused").
    pub state: State,

    /// True if super seeding is enabled.
    pub super_seeding: bool,

    /// Comma-concatenated tag list of the torrent.
    pub tags: String,

    /// Total active time (seconds).
    pub time_active: i64,

    /// Total size (bytes) of all files in this torrent (including unselected ones).
    pub total_size: u64,

    /// The first tracker with working status. Returns an empty string if no tracker is working.
    pub tracker: String,

    /// Torrent upload speed limit (bytes/s). `-1` if unlimited.
    pub up_limit: i64,

    /// Total amount of data uploaded.
    pub uploaded: u64,

    /// Amount of data uploaded in the current session.
    pub uploaded_session: u64,

    /// Torrent upload speed (bytes/s).
    pub upspeed: u64,
}

/// Represents the various states a torrent can be in.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum State {
    /// Some error occurred, applies to paused torrents.
    Error,

    /// Torrent data files are missing.
    MissingFiles,

    /// Torrent is being seeded and data is being transferred.
    Uploading,

    /// Torrent is paused and has finished downloading.
    PausedUP,

    /// Queuing is enabled and torrent is queued for upload.
    QueuedUP,

    /// Torrent is being seeded, but no connections were made.
    StalledUP,

    /// Torrent has finished downloading and is being checked.
    CheckingUP,

    /// Torrent is forced to uploading and ignores queue limits.
    ForcedUP,

    /// Torrent is allocating disk space for download.
    Allocating,

    /// Torrent is being downloaded and data is being transferred.
    Downloading,

    /// Torrent has just started downloading and is fetching metadata.
    MetaDL,

    /// Torrent is paused and has NOT finished downloading.
    PausedDL,

    /// Queuing is enabled and torrent is queued for download.
    QueuedDL,

    /// Torrent is being downloaded, but no connections were made.
    StalledDL,

    /// Torrent is being checked but has NOT finished downloading.
    CheckingDL,

    /// Torrent is forced to downloading and ignores queue limits.
    ForcedDL,

    /// Checking resume data on qBt startup.
    CheckingResumeData,

    /// Torrent is moving to another location.
    Moving,

    /// Unknown status.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QBittorrentClientOptions;
    use log::trace;
    use rogue_config::{OptionsProvider, YamlOptionsProvider};
    use rogue_logging::{Error, LoggerBuilder};

    #[tokio::test]
    async fn get_torrents() -> Result<(), Error> {
        // Arrange
        let _ = LoggerBuilder::new()
            .with_exclude_filter("reqwest".to_owned())
            .with_exclude_filter("cookie".to_owned())
            .create();
        let options: QBittorrentClientOptions = YamlOptionsProvider::get()?;
        let mut client = QBittorrentClient::from_options(options);
        let filters = FilterOptions {
            limit: Some(20),
            ..FilterOptions::default()
        };

        // Act
        let response = client.login().await?;
        trace!("{response:?}");
        let response = client.get_torrents(filters).await?;
        trace!("{}", response.to_json_pretty());

        // Assert
        let result = response.get_result("get_torrents")?;
        assert!(!result.is_empty());
        Ok(())
    }
}
