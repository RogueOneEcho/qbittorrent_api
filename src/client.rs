use async_trait::async_trait;
use std::path::PathBuf;

use crate::add_torrent::{AddTorrentAction, AddTorrentOptions};
use crate::get_torrents::{FilterOptions, Torrent};
#[cfg(test)]
use crate::{QBittorrentClientFactory, QBittorrentClientOptions};
use crate::{QBittorrentClientTrait, Response, Status};
use colored::Colorize;
use log::*;
use reqwest::cookie::Jar;
use reqwest::{Client, Method};
use rogue_logging::Failure;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::SystemTime;
use thiserror::Error;
use tower::limit::RateLimit;
use tower::{Service, ServiceExt};

/// A client for the qBittorrent API
///
/// Created by a [`QBittorrentClientFactory`]
pub struct QBittorrentClient {
    pub host: String,
    pub username: String,
    pub password: String,
    pub cookies: Arc<Jar>,
    pub client: RateLimit<Client>,
}

impl QBittorrentClient {
    #[cfg(test)]
    pub(crate) fn from_options(options: QBittorrentClientOptions) -> QBittorrentClient {
        let factory = QBittorrentClientFactory { options };
        factory.create()
    }

    pub(crate) async fn request<T: Serialize>(
        &mut self,
        method: Method,
        endpoint: &str,
        data: T,
    ) -> Result<reqwest::Response, Failure<ClientAction>> {
        trace!("{} request {method} {endpoint}", "Sending".bold());
        let url = format!("{}/api/v2{endpoint}", self.host);
        let request_builder = self.client.get_ref().request(method.clone(), url.clone());
        let request = match method {
            Method::GET => Ok(request_builder.query(&data)),
            Method::POST => Ok(request_builder.form(&data)),
            _ => {
                return Err(Failure::from_action(ClientAction::BuildRequest)
                    .with("method", method.to_string())
                    .with("endpoint", endpoint))
            }
        }?;
        let request = request
            .build()
            .map_err(Failure::wrap(ClientAction::BuildRequest))?;
        let start = SystemTime::now();
        let result = self
            .client
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
        result.map_err(Failure::wrap(ClientAction::SendRequest))
    }
}

pub(crate) async fn handle_status_response(
    method: &Method,
    endpoint: &str,
    response: reqwest::Response,
) -> Result<Status, Failure<ClientAction>> {
    let status_code = response.status().as_u16();
    let text = response.text().await.map_err(|e| {
        Failure::new(ClientAction::ReadResponseBody, e)
            .with("method", method.to_string())
            .with("endpoint", endpoint)
            .with("status_code", status_code.to_string())
    })?;
    Ok(Status::from(text.as_str()))
}

pub(crate) async fn deserialize_response<T: DeserializeOwned>(
    method: &Method,
    endpoint: &str,
    response: reqwest::Response,
) -> Result<Response<T>, Failure<ClientAction>> {
    let status_code = response.status().as_u16();
    let json = response.text().await.map_err(|e| {
        Failure::new(ClientAction::ReadResponseBody, e)
            .with("method", method.to_string())
            .with("endpoint", endpoint)
            .with("status_code", status_code.to_string())
    })?;
    match serde_json::from_str::<T>(&json) {
        Ok(result) => Ok(Response {
            status_code: Some(status_code),
            result: Some(result),
        }),
        Err(e) => {
            trace!("{json}");
            Err(Failure::new(ClientAction::DeserializeResponse, e)
                .with("method", method.to_string())
                .with("endpoint", endpoint)
                .with("status_code", status_code.to_string()))
        }
    }
}

#[async_trait]
impl QBittorrentClientTrait for QBittorrentClient {
    async fn login(&mut self) -> Result<Status, Failure<ClientAction>> {
        QBittorrentClient::login(self).await
    }
    async fn get_torrents(
        &mut self,
        filters: FilterOptions,
    ) -> Result<Response<Vec<Torrent>>, Failure<ClientAction>> {
        QBittorrentClient::get_torrents(self, filters).await
    }
    async fn add_torrent(
        &mut self,
        options: AddTorrentOptions,
        torrent: PathBuf,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>> {
        QBittorrentClient::add_torrent(self, options, torrent).await
    }
    async fn add_torrents(
        &mut self,
        options: AddTorrentOptions,
        torrents: Vec<PathBuf>,
    ) -> Result<Response<bool>, Failure<AddTorrentAction>> {
        QBittorrentClient::add_torrents(self, options, torrents).await
    }
}

/// Errors returned by [`QBittorrentClient`] request operations
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ClientAction {
    #[error("build request")]
    BuildRequest,
    #[error("send request")]
    SendRequest,
    #[error("read response body")]
    ReadResponseBody,
    #[error("deserialize response")]
    DeserializeResponse,
    #[error("validate response")]
    ValidateResponse,
}
