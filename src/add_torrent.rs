//! Torrent upload via multipart form requests.

use crate::{QBittorrentClient, Response, Status};
use colored::Colorize;
use log::{debug, trace};
use reqwest::Method;
use reqwest::multipart::{Form, Part};
use rogue_logging::Failure;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::SystemTime;
use thiserror::Error;
use tower::{Service, ServiceExt};

impl QBittorrentClient {
    /// Add torrent from file
    ///
    /// # See Also
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#add-new-torrent>
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#add-new-torrent>
    pub async fn add_torrent(
        &self,
        options: AddTorrentOptions,
        torrent: PathBuf,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>> {
        self.add_torrents(options, vec![torrent]).await
    }

    /// Add torrents from file
    ///
    /// # See Also
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#add-new-torrent>
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#add-new-torrent>
    pub async fn add_torrents(
        &self,
        options: AddTorrentOptions,
        torrents: Vec<PathBuf>,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>> {
        self.ensure_login()
            .await
            .map_err(Failure::wrap(AddTorrentAction::Login))?;
        let options_retry = options.clone();
        let torrents_retry = torrents.clone();
        let response = self.send_add_torrents(options, torrents).await?;
        if response.status().as_u16() == 403 {
            debug!(
                "{} 403 response for add_torrents, re-authenticating",
                "Received".bold()
            );
            let status = self
                .login()
                .await
                .map_err(Failure::wrap(AddTorrentAction::Login))?;
            if status != Status::Success {
                return Err(Failure::from_action(AddTorrentAction::Login)
                    .with("status", format!("{status:?}")));
            }
            let response = self
                .send_add_torrents(options_retry, torrents_retry)
                .await?;
            let status = response.status();
            return Ok(Response {
                status_code: Some(status.as_u16()),
                result: Some(status.is_success()),
            });
        }
        let status = response.status();
        Ok(Response {
            status_code: Some(status.as_u16()),
            result: Some(status.is_success()),
        })
    }

    async fn send_add_torrents(
        &self,
        options: AddTorrentOptions,
        torrents: Vec<PathBuf>,
    ) -> Result<reqwest::Response, Failure<AddTorrentAction>> {
        let method = Method::POST;
        let endpoint = "/torrents/add";
        let url = format!("{}/api/v2{endpoint}", self.host);
        let mut client = self.client.lock().await;
        let request = client
            .get_ref()
            .request(method.clone(), url.clone())
            .multipart(options.to_form(torrents)?)
            .build()
            .map_err(Failure::wrap(AddTorrentAction::BuildRequest))?;
        let start = SystemTime::now();
        let result = client
            .ready()
            .await
            .expect("rate limiter should be available")
            .call(request)
            .await;
        let elapsed = start
            .elapsed()
            .expect("elapsed should not fail")
            .as_secs_f64();
        trace!("{} response after {elapsed:.3}", "Received".bold());
        result.map_err(Failure::wrap(AddTorrentAction::SendRequest))
    }
}

/// Options for adding torrents to qBittorrent.
///
/// - Intentionally exposes only a subset of the parameters accepted by `POST /torrents/add`.
/// - Fields are added on demand. To request one, open an issue describing your use case.
///
/// # See Also
/// - <https://github.com/RogueOneEcho/qbittorrent_api/issues>
/// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#add-new-torrent>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-4.1.0/src/webui/api/torrentscontroller.cpp#L453>
/// - <https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/webui/api/torrentscontroller.cpp#L693>
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AddTorrentOptions {
    /// Path to the downloads folder the torrent content is stored in
    pub save_path: Option<String>,
    /// Category for the torrent
    pub category: Option<String>,
    /// Tags for the torrent, split by ','
    pub tags: Option<Vec<String>>,
    /// Skip hash checking
    pub skip_checking: Option<bool>,
    /// Add torrents in the paused state.
    pub paused: Option<bool>,
    /// Create the root folder.
    pub root_folder: Option<bool>,
    /// Rename torrent
    pub rename: Option<String>,
    /// Set torrent upload speed limit. Unit in bytes/second
    pub up_limit: Option<usize>,
    /// Set torrent download speed limit. Unit in bytes/second
    pub dl_limit: Option<usize>,
    /// Set torrent share ratio limit
    pub ratio_limit: Option<f32>,
    /// Set torrent seeding time limit. Unit in minutes
    pub seeding_time_limit: Option<usize>,
    /// Whether Automatic Torrent Management should be used
    pub automatic_torrent_management: Option<bool>,
    /// Enable sequential download
    pub sequential_download: Option<bool>,
    /// Prioritize first and last pieces
    pub first_last_piece_priority: Option<bool>,
}

impl AddTorrentOptions {
    /// Build a multipart form from these options and torrent files.
    pub fn to_form(self, torrents: Vec<PathBuf>) -> Result<Form, Failure<AddTorrentAction>> {
        let mut form = Form::new();
        for torrent in torrents {
            form = form.part("torrents", get_torrent_part(torrent)?);
        }
        if let Some(save_path) = &self.save_path {
            form = form.text("savepath", save_path.clone());
        }
        if let Some(category) = &self.category {
            form = form.text("category", category.clone());
        }
        if let Some(tags) = &self.tags {
            form = form.text("tags", tags.join(","));
        }
        if let Some(skip_checking) = self.skip_checking {
            form = form.text("skip_checking", skip_checking.to_string());
        }
        if let Some(paused) = self.paused {
            // `stopped` replaces `paused` in `v5.0.0`
            // https://github.com/qbittorrent/qBittorrent/issues/21561#issuecomment-2558072321
            form = form.text("paused", paused.to_string());
            form = form.text("stopped", paused.to_string());
        }
        if let Some(root_folder) = self.root_folder {
            form = form.text("root_folder", root_folder.to_string());
        }
        if let Some(rename) = &self.rename {
            form = form.text("rename", rename.clone());
        }
        if let Some(up_limit) = self.up_limit {
            form = form.text("upLimit", up_limit.to_string());
        }
        if let Some(dl_limit) = self.dl_limit {
            form = form.text("dlLimit", dl_limit.to_string());
        }
        if let Some(ratio_limit) = self.ratio_limit {
            form = form.text("ratioLimit", ratio_limit.to_string());
        }
        if let Some(seeding_time_limit) = self.seeding_time_limit {
            form = form.text("seedingTimeLimit", seeding_time_limit.to_string());
        }
        if let Some(automatic_torrent_management) = self.automatic_torrent_management {
            form = form.text("autoTMM", automatic_torrent_management.to_string());
        }
        if let Some(sequential_download) = self.sequential_download {
            form = form.text("sequentialDownload", sequential_download.to_string());
        }
        if let Some(first_last_piece_priority) = self.first_last_piece_priority {
            form = form.text("firstLastPiecePrio", first_last_piece_priority.to_string());
        }
        Ok(form)
    }
}

fn get_torrent_part(path: PathBuf) -> Result<Part, Failure<AddTorrentAction>> {
    let mut file =
        File::open(&path).map_err(Failure::wrap_with_path(AddTorrentAction::OpenFile, &path))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(Failure::wrap_with_path(AddTorrentAction::ReadFile, &path))?;
    let filename = path
        .file_name()
        .expect("file should have a name")
        .to_string_lossy()
        .to_string();
    Ok(Part::bytes(buffer).file_name(filename))
}

/// Errors returned by torrent add operations
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum AddTorrentAction {
    #[error("open torrent file")]
    OpenFile,
    #[error("read torrent file")]
    ReadFile,
    #[error("build request")]
    BuildRequest,
    #[error("send request")]
    SendRequest,
    #[error("login")]
    Login,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QBittorrentClientOptions;
    use crate::tests::init_logger;
    use log::trace;
    use rogue_config::OptionsProvider;
    use rogue_config::YamlOptionsProvider;
    use std::error::Error;

    #[tokio::test]
    #[ignore = "integration test requiring API credentials"]
    async fn add_torrents() -> Result<(), Box<dyn Error>> {
        // Arrange
        init_logger();
        let options: QBittorrentClientOptions =
            YamlOptionsProvider::get().map_err(|e| e.to_string())?;
        let client = QBittorrentClient::from_options(options);
        let torrents = vec![
            PathBuf::from("/srv/shared/tests/example-1.torrent"),
            PathBuf::from("/srv/shared/tests/example-2.torrent"),
        ];
        let options = AddTorrentOptions {
            save_path: Some("/srv/shared/tests".to_owned()),
            category: Some("example".to_owned()),
            paused: Some(true),
            skip_checking: Some(true),
            ..AddTorrentOptions::default()
        };

        // Act
        let response = client.add_torrents(options, torrents).await?;
        trace!("{}", response.to_json_pretty());

        // Assert
        let _result = response.get_result("add_torrents")?;
        Ok(())
    }
}
