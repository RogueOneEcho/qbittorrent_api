use serde::{Deserialize, Serialize};

/// # See also
/// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#login>
#[allow(clippy::doc_markdown)]
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct QBittorrentClientOptions {
    /// QBittorrent WebUI API host including port but without protocol or password
    ///
    /// # Examples
    /// - `localhost`
    /// - `example.com`
    /// - `example.com:3000`
    /// - `127.0.0.1`
    pub host: String,

    /// QBittorrent WebUI API username
    pub username: String,

    /// QBittorrent WebUI API password
    pub password: String,

    /// User agent
    pub user_agent: Option<String>,

    /// Number of requests permitted per `rate_limit_duration`
    pub rate_limit_count: Option<usize>,

    /// Duration before rate limit is reset
    pub rate_limit_duration: Option<usize>,
}
