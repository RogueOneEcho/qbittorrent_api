use crate::client::DOMAIN;
use crate::{QBittorrentClient, Response};
use colored::Colorize;
use log::trace;
use reqwest::multipart::{Form, Part};
use reqwest::Method;
use rogue_logging::Error;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::SystemTime;

impl QBittorrentClient {
    /// Add torrents from file
    ///
    /// # See Also
    /// - <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#add-new-torrent>
    pub async fn add_torrent(&mut self, torrent: Torrent) -> Result<Response<bool>, Error> {
        let method = Method::POST;
        let endpoint = "/torrents/add";
        let url = format!("{}/api/v2{endpoint}", self.host);
        let client = self.wait_for_client().await;
        let start = SystemTime::now();
        let request = client
            .request(method.clone(), url.clone())
            .multipart(torrent.to_form()?);
        let result = request.send().await;
        let elapsed = start
            .elapsed()
            .expect("elapsed should not fail")
            .as_secs_f64();
        trace!("{} response after {elapsed:.3}", "Received".bold());
        let response = result.map_err(|e| Error {
            action: format!("send {method} {endpoint} request"),
            domain: Some(DOMAIN.to_owned()),
            message: e.to_string(),
            ..Error::default()
        })?;
        let status = response.status();
        Ok(Response {
            status_code: Some(status.as_u16()),
            result: Some(status.is_success()),
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Torrent {
    /// Path of the .torrent file
    pub path: PathBuf,

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

    /// Enable sequential download
    pub first_last_piece_priority: Option<bool>,
}

impl Torrent {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_form(self) -> Result<Form, Error> {
        let mut form = Form::new().part("torrents", self.get_torrent_part()?);
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
            form = form.text("paused", paused.to_string());
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

    fn get_torrent_part(&self) -> Result<Part, Error> {
        let action = "add torrent";
        let mut file = File::open(&self.path).map_err(|e| Error {
            action: action.to_owned(),
            message: e.to_string(),
            ..Error::default()
        })?;
        let mut buffer = Vec::new();
        let _size = file.read_to_end(&mut buffer).map_err(|e| Error {
            action: action.to_owned(),
            message: e.to_string(),
            ..Error::default()
        })?;
        let filename = self
            .path
            .file_name()
            .expect("file should have a name")
            .to_string_lossy()
            .to_string();
        Ok(Part::bytes(buffer).file_name(filename))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QBittorrentClientOptions;
    use log::trace;
    use rogue_config::OptionsProvider;
    use rogue_config::YamlOptionsProvider;
    use rogue_logging::{Error, LoggerBuilder};

    #[tokio::test]
    #[ignore]
    async fn add_torrents() -> Result<(), Error> {
        // Arrange
        let _ = LoggerBuilder::new()
            .with_exclude_filter("reqwest".to_owned())
            .with_exclude_filter("cookie".to_owned())
            .create();
        let options: QBittorrentClientOptions = YamlOptionsProvider::get()?;
        let mut client = QBittorrentClient::from_options(options);
        let torrent = Torrent {
            path: PathBuf::from("/srv/shared/tests/example-1.torrent"),
            save_path: Some("/srv/shared/tests".to_owned()),
            category: Some("uploaded".to_owned()),
            ..Torrent::default()
        };

        // Act
        let response = client.login().await?;
        trace!("{response:?}");
        let response = client.add_torrent(torrent).await?;
        trace!("{}", response.to_json_pretty());

        // Assert
        let _result = response.get_result("add_torrents")?;
        Ok(())
    }
}
