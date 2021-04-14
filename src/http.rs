use crate::context::{Metrics, SyncContext};
use async_std::future;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use surf::http::convert::{Deserialize, DeserializeOwned, Serialize};
use surf::StatusCode;
use url::Url;

#[derive(Serialize, Deserialize)]
pub struct HttpError {
    pub detail: String,
    pub status: u16,
    pub title: String,
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "status: {}, title: {}, detail: {}",
            self.status, self.title, self.detail
        )
    }
}

impl From<surf::Error> for HttpError {
    fn from(e: surf::Error) -> Self {
        HttpError {
            detail: e.to_string(),
            status: u16::from(e.status()),
            title: e.to_string(),
        }
    }
}

impl HttpError {
    pub fn new(status: u16, title: String, detail: String) -> HttpError {
        HttpError {
            detail,
            status,
            title,
        }
    }
    pub fn from_err(e: impl Error) -> HttpError {
        HttpError {
            detail: e.to_string(),
            status: 0,
            title: "".to_string(),
        }
    }
    pub fn unknown_error(msg: String) -> HttpError {
        HttpError {
            detail: msg,
            status: 666,
            title: "Unknown error".to_string(),
        }
    }
    pub fn timeout(msg: String) -> HttpError {
        HttpError {
            detail: msg,
            status: 667,
            title: "Timeout".to_string(),
        }
    }
}

pub async fn _http_get(
    url: &Url,
    timeout: Duration,
    client: &surf::Client,
    _sync: SyncContext,
) -> Result<String, HttpError> where
{
    match future::timeout(timeout, client.get(url.as_str())).await {
        Ok(res) => match res {
            Ok(mut response) => Ok(response.body_string().await.unwrap()),
            Err(e) => Err(HttpError::unknown_error(e.to_string())),
        },
        Err(e) => Err(HttpError::unknown_error(e.to_string())),
    }
}

pub async fn http_post<'a, T>(
    url: &Url,
    timeout: Duration,
    payload: impl Serialize,
    client: &surf::Client,
    sync: SyncContext,
) -> Result<T, HttpError>
where
    T: Serialize + DeserializeOwned,
{
    sync.http_rate_limiter().await;
    let body = surf::Body::from_json(&payload).unwrap();
    let result = match future::timeout(timeout, client.post(url.as_str()).body(body)).await {
        Ok(res) => match res {
            Ok(mut response) => {

                if response.status() == StatusCode::Ok {
                    sync.metrics_sender.send(Metrics::new200()).await.unwrap();
                } else if response.status() == StatusCode::NotFound {
                    sync.metrics_sender.send(Metrics::new404()).await.unwrap();
                } else if response.status() == StatusCode::TooManyRequests {
                    sync.metrics_sender.send(Metrics::new429()).await.unwrap();
                } else if response.status() == StatusCode::UnprocessableEntity {
                    sync.metrics_sender.send(Metrics::new422()).await.unwrap();
                } else if response.status() == StatusCode::Conflict {
                    sync.metrics_sender.send(Metrics::new409()).await.unwrap();
                } else if response.status().is_server_error() {
                    sync.metrics_sender.send(Metrics::new50x()).await.unwrap();
                } else {
                    println!("wtf {}", response.body_string().await.unwrap());
                    sync.metrics_sender
                        .send(Metrics::new_other())
                        .await
                        .unwrap();
                }


                if response.status().is_success() {
                    let data = response.body_bytes().await?;
                    Ok(serde_json::from_slice(&data).map_err(HttpError::from_err)?)
                } else {
                    let http_error = match response.body_json().await {
                        Ok(j) => Err(j),
                        Err(e) => Err(HttpError::new(
                            u16::from(response.status()),
                            e.to_string(),
                            e.to_string(),
                        )),
                    };
                    http_error
                }
            }
            Err(e) => {

                sync.metrics_sender
                    .send(Metrics::new_other())
                    .await
                    .unwrap();

                Err(HttpError::unknown_error(e.to_string()))
            }
        },
        Err(e) => {

            sync.metrics_sender
                .send(Metrics::new_other())
                .await
                .unwrap();
            Err(HttpError::timeout(e.to_string()))
        }
    };
    result
}
