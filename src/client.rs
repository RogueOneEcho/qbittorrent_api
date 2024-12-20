#[cfg(test)]
use crate::{QBittorrentClientFactory, QBittorrentClientOptions};
use crate::{Response, Status};
use colored::Colorize;
use log::*;
use reqwest::cookie::Jar;
use reqwest::{Client, Method};
use rogue_logging::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tower::limit::RateLimit;
use tower::ServiceExt;

pub(crate) const DOMAIN: &str = "qBittorrent API";

/// A client for the Deluge API
///
/// Created by an [`QBittorrentClientFactory`]
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
    ) -> Result<reqwest::Response, Error> {
        trace!("{} request {method} {endpoint}", "Sending".bold());
        let url = format!("{}/api/v2{endpoint}", self.host);
        let client = self.wait_for_client().await;
        let start = SystemTime::now();
        let request = client.request(method.clone(), url.clone());
        let request = match method {
            Method::GET => Ok(request.query(&data)),
            Method::POST => Ok(request.form(&data)),
            _ => {
                return Err(Error {
                    action: format!("send {method} {endpoint} request"),
                    domain: Some(DOMAIN.to_owned()),
                    message: format!("Method {method} is not supported"),
                    ..Error::default()
                })
            }
        }?;
        let result = request.send().await;
        let elapsed = start
            .elapsed()
            .expect("elapsed should not fail")
            .as_secs_f64();
        trace!("{} response after {elapsed:.3}", "Received".bold());
        result.map_err(|e| Error {
            action: format!("send {method} {endpoint} request"),
            domain: Some(DOMAIN.to_owned()),
            message: e.to_string(),
            ..Error::default()
        })
    }

    pub(crate) async fn wait_for_client(&mut self) -> &Client {
        let start = SystemTime::now();
        let client = self
            .client
            .ready()
            .await
            .expect("client should be available")
            .get_ref();
        let duration = start.elapsed().expect("duration should not fail");
        if duration > Duration::from_millis(200) {
            trace!(
                "{} {:.3} for rate limiter",
                "Waited".bold(),
                duration.as_secs_f64()
            );
        }
        client
    }
}

pub(crate) async fn handle_status_response(
    method: Method,
    endpoint: &str,
    response: reqwest::Response,
) -> Result<Status, Error> {
    let status_code = Some(response.status().as_u16());
    let text = response.text().await.map_err(|e| Error {
        action: format!("get response body of {method} {endpoint} request"),
        domain: Some(DOMAIN.to_owned()),
        message: e.to_string(),
        status_code,
        ..Error::default()
    })?;
    Ok(Status::from(text.as_str()))
}

pub(crate) async fn deserialize_response<T: DeserializeOwned>(
    method: Method,
    endpoint: &str,
    response: reqwest::Response,
) -> Result<Response<T>, Error> {
    let status_code = Some(response.status().as_u16());
    let json = response.text().await.map_err(|e| Error {
        action: format!("get response body of {method} {endpoint} request"),
        domain: Some(DOMAIN.to_owned()),
        message: e.to_string(),
        status_code,
        ..Error::default()
    })?;
    match serde_json::from_str::<T>(&json) {
        Ok(result) => Ok(Response {
            status_code,
            result: Some(result),
        }),
        Err(e) => {
            trace!("{json}");
            Err(Error {
                action: format!("deserialize response of {DOMAIN} {method} {endpoint} request"),
                domain: Some("deserialization".to_owned()),
                message: e.to_string(),
                status_code,
                ..Error::default()
            })
        }
    }
}
