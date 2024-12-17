use crate::Response;
#[cfg(test)]
use crate::{QBittorrentClientFactory, QBittorrentClientOptions};
use colored::Colorize;
use log::*;
use reqwest::cookie::Jar;
use reqwest::{Client, Method};
use rogue_logging::Error;
use serde::de::DeserializeOwned;
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

    pub(crate) async fn request(
        &mut self,
        method: Method,
        endpoint: &str,
        data: &[(&str, &str)],
    ) -> Result<reqwest::Response, Error> {
        trace!("{} request {method} /{endpoint}", "Sending".bold());
        let url = format!("{}/{endpoint}", self.host);
        let client = self.wait_for_client().await;
        let start = SystemTime::now();
        let result = client
            .request(method.clone(), url)
            .query(&data)
            .send()
            .await;
        let elapsed = start
            .elapsed()
            .expect("elapsed should not fail")
            .as_secs_f64();
        trace!("{} response after {elapsed:.3}", "Received".bold());
        result.map_err(|e| Error {
            action: format!("send {method} /{endpoint} request"),
            domain: Some(DOMAIN.to_owned()),
            message: e.to_string(),
            ..Error::default()
        })
    }

    async fn wait_for_client(&mut self) -> &Client {
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

pub(crate) async fn deserialize_response<T: DeserializeOwned>(
    method: Method,
    endpoint: &str,
    response: reqwest::Response,
) -> Result<Response<T>, Error> {
    let status_code = Some(response.status().as_u16());
    let json = response.text().await.map_err(|e| Error {
        action: format!("get response body of {method} /{endpoint} request"),
        domain: Some(DOMAIN.to_owned()),
        message: e.to_string(),
        status_code,
        ..Error::default()
    })?;
    match serde_json::from_str::<Response<T>>(&json) {
        Ok(mut response) => {
            response.status_code = status_code;
            Ok(response)
        }
        Err(e) => {
            trace!("{json}");
            Err(Error {
                action: format!("deserialize response of {DOMAIN} {method} /{endpoint} request"),
                domain: Some("deserialization".to_owned()),
                message: e.to_string(),
                status_code,
                ..Error::default()
            })
        }
    }
}
