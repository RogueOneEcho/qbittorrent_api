//! Torrent listing, filtering, and response models.

use crate::QBittorrentClient;
use crate::Response;
use crate::client::{ClientAction, deserialize_response};
use reqwest::Method;
use rogue_logging::Failure;
use serde::{Deserialize, Serialize};

impl QBittorrentClient {
    /// Get all torrents matching the filter
    ///
    /// # See Also
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#get-torrent-list>
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#get-torrent-list>
    /// - <https://github.com/qbittorrent/qBittorrent/blob/release-4.1.0/src/webui/api/torrentscontroller.cpp#L157>
    /// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/webui/api/torrentscontroller.cpp#L293>
    pub async fn get_torrents(
        &self,
        filters: FilterOptions,
    ) -> Result<Response<Vec<Torrent>>, Failure<ClientAction>> {
        let method = Method::GET;
        let endpoint = "/torrents/info";
        let response = self
            .request_with_login(method.clone(), endpoint, &filters)
            .await?;
        deserialize_response::<Vec<Torrent>>(&method, endpoint, response).await
    }
}

/// Query parameters for filtering the torrent list.
///
/// # See Also
/// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#get-torrent-list>
/// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#get-torrent-list>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-4.1.0/src/webui/api/torrentscontroller.cpp#L157>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/webui/api/torrentscontroller.cpp#L293>
#[derive(Debug, Default, Deserialize, Serialize)]
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
///
/// # See Also
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-4.1.0/src/webui/api/torrentscontroller.cpp#L151>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/base/torrentfilter.cpp#L89>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterState {
    All,
    Downloading,
    Seeding,
    Completed,
    Active,
    Inactive,
    /// v4.1 only, replaced by [`Stopped`] in v5.0
    Paused,
    /// v4.1 only, replaced by [`Running`] in v5.0
    Resumed,
    /// Added in v4.6
    Stalled,
    /// Added in v4.6
    StalledUploading,
    /// Added in v4.6
    StalledDownloading,
    /// Added in v4.6
    Checking,
    /// Added in v4.6
    Moving,
    /// Added in v4.6
    Errored,
    /// Replaces [`Paused`] in v5.0
    Stopped,
    /// Replaces [`Resumed`] in v5.0
    Running,
}

/// Represents detailed information about a torrent.
///
/// # See Also
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-4.1.0/src/webui/api/serialize/serialize_torrent.cpp#L86>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-4.1.0/src/webui/api/serialize/serialize_torrent.h#L39>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/webui/api/serialize/serialize_torrent.cpp#L110>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/webui/api/serialize/serialize_torrent.h#L42>
#[derive(Clone, Debug, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors the qBittorrent API response"
)]
pub struct Torrent {
    /// Time (Unix Epoch) when the torrent was added to the client.
    pub added_on: i64,

    /// Amount of data left to download (bytes).
    pub amount_left: u64,

    /// Whether this torrent is managed by Automatic Torrent Management.
    pub auto_tmm: bool,

    /// Category of the torrent.
    pub category: String,

    /// Amount of transfer data completed (bytes).
    pub completed: u64,

    /// Time (Unix Epoch) when the torrent completed.
    pub completion_on: i64,

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

    /// Torrent identifier, used as the canonical ID throughout the `WebUI` API.
    ///
    /// - For v1-only torrents: the SHA1 v1 info-hash.
    /// - For `BitTorrent` v2 and hybrid torrents (qBittorrent 4.4+ built against
    ///   libtorrent 2.x): the SHA256 v2 info-hash truncated to its first 20 bytes,
    ///   **not** the v1 info-hash. The JSON value comes from
    ///   `Torrent::id().toString()`, which calls `InfoHash::toTorrentID()` and
    ///   in turn libtorrent's `info_hash_t::get_best()`.
    ///
    /// Use [`infohash_v1`](Self::infohash_v1) and [`infohash_v2`](Self::infohash_v2)
    /// (added in v4.4) when the unambiguous v1 or v2 hash is required.
    ///
    /// Despite the JSON field name, qBittorrent treats this as a torrent ID
    /// rather than an info-hash; the upstream `KEY_TORRENT_ID` constant is
    /// marked `TODO: Rename it to 'id'`.
    ///
    /// # See Also
    /// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/webui/api/serialize/serialize_torrent.h#L41-L42>
    /// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/base/bittorrent/torrent.cpp#L58-L61>
    /// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/base/bittorrent/infohash.cpp#L80-L87>
    /// - <https://github.com/arvidn/libtorrent/blob/v2.0.12/include/libtorrent/info_hash.hpp#L110-L115>
    pub hash: String,

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

    /// Per-torrent share ratio limit set by the user.
    ///
    /// `-1` uses the global limit, `-2` means no limit.
    pub ratio_limit: f64,

    /// Path where this torrent's data is stored.
    pub save_path: String,

    /// Per-torrent seeding time limit in seconds set by the user.
    ///
    /// `-1` uses the global limit, `-2` means no limit.
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

    /// Percentage of file pieces currently available.
    ///
    /// Added in v4.5
    #[serde(default)]
    pub availability: Option<f64>,

    /// Absolute path of torrent content.
    /// Root path for multifile torrents, absolute file path for singlefile torrents.
    ///
    /// Added in v4.5
    #[serde(default)]
    pub content_path: Option<String>,

    /// Download path for the torrent.
    ///
    /// Added in v4.5
    #[serde(default)]
    pub download_path: Option<String>,

    /// Maximum inactive seeding time (seconds) until torrent is stopped.
    ///
    /// Added in v4.6
    #[serde(default)]
    pub max_inactive_seeding_time: Option<i64>,

    /// Per-torrent inactive seeding time limit in seconds.
    ///
    /// `-1` uses the global limit, `-2` means no limit.
    ///
    /// Added in v4.6
    #[serde(default)]
    pub inactive_seeding_time_limit: Option<i64>,

    /// V1 info hash of the torrent.
    ///
    /// Added in v4.4
    #[serde(default)]
    pub infohash_v1: Option<String>,

    /// V2 info hash of the torrent.
    ///
    /// Added in v4.4
    #[serde(default)]
    pub infohash_v2: Option<String>,

    /// Torrent elapsed time while complete (seconds).
    ///
    /// Added in v4.5
    #[serde(default)]
    pub seeding_time: Option<i64>,

    /// Number of trackers for this torrent.
    ///
    /// Added in v4.5
    #[serde(default)]
    pub trackers_count: Option<u64>,

    /// Torrent comment.
    ///
    /// Added in v5.0
    #[serde(default)]
    pub comment: Option<String>,

    /// True if torrent has metadata available.
    ///
    /// Added in v5.0
    ///
    /// Always `None` when the server is qui's reverse proxy, even on
    /// qBittorrent 5.0+. See <https://github.com/autobrr/qui/discussions/1754>.
    #[serde(default)]
    pub has_metadata: Option<bool>,

    /// Torrent popularity score.
    ///
    /// Added in v5.0
    #[serde(default)]
    pub popularity: Option<f64>,

    /// True if torrent is from a private tracker.
    ///
    /// Added in v5.0
    #[serde(default)]
    pub private: Option<bool>,

    /// Time until the next tracker reannounce (seconds).
    ///
    /// Added in v5.0
    #[serde(default)]
    pub reannounce: Option<i64>,

    /// Root path of the torrent.
    ///
    /// Added in v5.0
    ///
    /// Always `None` when the server is qui's reverse proxy, even on
    /// qBittorrent 5.0+. See <https://github.com/autobrr/qui/discussions/1754>.
    #[serde(default)]
    pub root_path: Option<String>,
}

#[cfg(feature = "mock")]
impl Torrent {
    /// Create a mock `Torrent` for testing
    #[must_use]
    pub fn mock() -> Self {
        Self {
            added_on: 1_700_000_000,
            amount_left: 0,
            auto_tmm: false,
            category: "music".to_owned(),
            completed: 104_857_600,
            completion_on: 1_700_000_120,
            dl_limit: 0,
            dlspeed: 0,
            downloaded: 104_857_600,
            downloaded_session: 0,
            eta: 8_640_000,
            f_l_piece_prio: false,
            force_start: false,
            hash: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_owned(),
            last_activity: 1_700_100_000,
            magnet_uri:
                "magnet:?xt=urn:btih:a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2&dn=Artist+-+Album"
                    .to_owned(),
            max_ratio: -1.0,
            max_seeding_time: -1,
            name: "Artist - Album [2023] [WEB FLAC]".to_owned(),
            num_complete: 12,
            num_incomplete: 2,
            num_leechs: 1,
            num_seeds: 0,
            priority: 0,
            progress: 1.0,
            ratio: 2.5,
            ratio_limit: -2.0,
            save_path: "/downloads".to_owned(),
            seeding_time_limit: -2,
            seen_complete: 1_700_050_000,
            seq_dl: false,
            size: 104_857_600,
            state: State::StalledUP,
            super_seeding: false,
            tags: String::new(),
            time_active: 86_520,
            total_size: 104_857_600,
            tracker: "https://tracker.example.com/announce".to_owned(),
            up_limit: 0,
            uploaded: 262_144_000,
            uploaded_session: 0,
            upspeed: 0,
            availability: Some(-1.0),
            content_path: Some("/downloads/Artist - Album [2023] [WEB FLAC]".to_owned()),
            download_path: None,
            max_inactive_seeding_time: None,
            inactive_seeding_time_limit: None,
            infohash_v1: None,
            infohash_v2: None,
            seeding_time: Some(86_400),
            trackers_count: None,
            comment: None,
            has_metadata: None,
            popularity: None,
            private: None,
            reannounce: None,
            root_path: None,
        }
    }
}

/// Represents the various states a torrent can be in.
///
/// # See Also
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-4.1.0/src/webui/api/serialize/serialize_torrent.cpp#L38>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/webui/api/serialize/serialize_torrent.cpp#L44>
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum State {
    /// Some error occurred, applies to paused torrents.
    Error,

    /// Torrent data files are missing.
    MissingFiles,

    /// Torrent is being seeded and data is being transferred.
    Uploading,

    /// Torrent is paused and has finished downloading.
    ///
    /// Replaced by [`StoppedUP`] in `v5.0.0`
    /// <https://github.com/qbittorrent/qBittorrent/issues/21561#issuecomment-2558072321>
    PausedUP,

    /// Torrent is stopped and has finished downloading.
    ///
    /// Replaces [`PausedUP`] in `v5.0.0`
    /// <https://github.com/qbittorrent/qBittorrent/issues/21561#issuecomment-2558072321>
    StoppedUP,

    /// Queuing is enabled and torrent is queued for upload.
    QueuedUP,

    /// Torrent is being seeded, but no connections were made.
    StalledUP,

    /// Torrent has finished downloading and is being checked.
    CheckingUP,

    /// Torrent is forced to uploading and ignores queue limits.
    ForcedUP,

    /// Torrent is allocating disk space for download.
    ///
    /// v4.1 only, removed in v5.0
    Allocating,

    /// Torrent is being downloaded and data is being transferred.
    Downloading,

    /// Torrent has just started downloading and is fetching metadata.
    MetaDL,

    /// Torrent is forced to downloading metadata and ignores queue limits.
    ///
    /// Added in v5.0
    ForcedMetaDL,

    /// Torrent is paused and has NOT finished downloading.
    ///
    /// Replaced by [`StoppedDL`] in `v5.0.0`
    /// <https://github.com/qbittorrent/qBittorrent/issues/21561#issuecomment-2558072321>
    PausedDL,

    /// Torrent is stopped and has NOT finished downloading.
    ///
    /// Replaces [`PausedDL`] in `v5.0.0`
    /// <https://github.com/qbittorrent/qBittorrent/issues/21561#issuecomment-2558072321>
    StoppedDL,

    /// Queuing is enabled and torrent is queued for download.
    QueuedDL,

    /// Torrent is being downloaded, but no connections were made.
    StalledDL,

    /// Torrent is being checked but has NOT finished downloading.
    CheckingDL,

    /// Torrent is forced to downloading and ignores queue limits.
    ForcedDL,

    /// Torrent is queued for checking.
    ///
    /// v4.1 only, removed in later versions
    QueuedForChecking,

    /// Checking resume data on qBt startup.
    CheckingResumeData,

    /// Torrent is moving to another location.
    Moving,

    /// Unknown status.
    Unknown,
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "indexing after length validation in integration tests"
)]
mod tests {
    use super::*;
    use crate::QBittorrentClientOptions;
    use crate::tests::init_logger;
    use insta::assert_yaml_snapshot;
    use log::trace;
    use rogue_config::{OptionsProvider, YamlOptionsProvider};

    use std::error::Error;

    const FIXTURE_V4: &str = include_str!("tests/fixtures/torrents_info_v4.json");
    const FIXTURE_V5: &str = include_str!("tests/fixtures/torrents_info_v5.json");

    #[tokio::test]
    #[ignore = "integration test requiring API credentials"]
    async fn get_torrents() -> Result<(), Box<dyn Error>> {
        // Arrange
        init_logger();
        let options: QBittorrentClientOptions =
            YamlOptionsProvider::get().map_err(|e| e.to_string())?;
        let client = QBittorrentClient::from_options(options);
        let filters = FilterOptions {
            limit: Some(20),
            category: Some("example".to_owned()),
            ..FilterOptions::default()
        };

        // Act
        let response = client.get_torrents(filters).await?;
        trace!("{}", response.to_json_pretty());

        // Assert
        let result = response.get_result("get_torrents")?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[test]
    fn deserialize_v4_fixture() {
        let torrents: Vec<Torrent> =
            serde_json::from_str(FIXTURE_V4).expect("v4 fixture should deserialize");
        assert_yaml_snapshot!(torrents);
    }

    #[test]
    fn deserialize_v5_fixture() {
        let torrents: Vec<Torrent> =
            serde_json::from_str(FIXTURE_V5).expect("v5 fixture should deserialize");
        assert_yaml_snapshot!(torrents);
    }

    #[test]
    fn serialize_filter_options() {
        let filters = FilterOptions {
            filter: Some(FilterState::Seeding),
            category: Some("music".to_owned()),
            limit: Some(10),
            ..FilterOptions::default()
        };
        let json = serde_json::to_value(&filters).expect("filters should serialize");
        assert_eq!(json["filter"], "seeding");
        assert_eq!(json["category"], "music");
        assert_eq!(json["limit"], 10);
        assert!(json.get("tag").is_none());
        assert!(json.get("sort").is_none());
    }
}
