use anyhow::Result;
use chrono::Utc;
use futures_util::Future;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, Secret};
use std::{
    fs,
    io::ErrorKind,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{interval, Interval};
use tracing::instrument;

use crate::lnd::LnGetInfo;

#[derive(Clone, Debug)]
pub struct LndManager {
    url: String,
    macaroon: Secret<String>,
    ping_freq_secs: u64,
    client: reqwest::Client,
    kill_signal: Arc<AtomicBool>,
}

impl LndManager {
    #[instrument(skip_all)]
    pub async fn run(self) -> Result<(), std::io::Error> {
        tokio::spawn(async move {
            let interval = interval(Duration::from_secs(self.ping_freq_secs));
            ping_node(self.clone(), interval)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
        Ok(())
    }
    #[instrument(skip(kill_signal, macaroonpath))]
    pub fn build(
        url: String,
        ping_freq_secs: u64,
        kill_signal: Arc<AtomicBool>,
        macaroonpath: String,
    ) -> Self {
        Self {
            url,
            ping_freq_secs,
            kill_signal,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("should be able to build reqwest client"),
            macaroon: build_macaroon(macaroonpath),
        }
    }
}
#[instrument(skip_all)]
async fn ping_node(lnd_manager: LndManager, mut interval: Interval) -> Result<(), anyhow::Error> {
    let kill_signal = lnd_manager.clone().kill_signal;
    loop {
        if kill_signal.load(Ordering::Relaxed) {
            return Ok(());
        }
        match call_node(lnd_manager.clone()).await {
            Ok(_) => {}
            Err(e) => tracing::error!("error calling node: {}", e),
        }
        interval.tick().await;
    }
}
#[instrument(skip_all)]
async fn call_node(lnd_manager: LndManager) -> Result<(), anyhow::Error> {
    let command = "/v1/getinfo";
    let url = lnd_manager.url + command;
    tracing::info!("get url: {}", url);
    let headers = build_headers(lnd_manager.macaroon);
    let response = lnd_manager.client.get(url).headers(headers).send().await?;

    if response.status() != reqwest::StatusCode::OK {
        //NOTE: we just want to log the error, not shut down pinging
        tracing::error!("error pinging lnd node {:?}", response.text().await?);
        return Ok(());
    }
    let now = Utc::now();
    tracing::info!("ping was successfull at: {}", now.to_rfc3339());
    let data = response.text().await?;
    let lnd_get_info: LnGetInfo = serde_json::from_str(&data)?;
    tracing::info!("current state: {:?}", lnd_get_info);
    Ok(())
}
#[instrument(skip_all)]
fn build_headers(macaroon: Secret<String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let type_header = HeaderValue::from_str("application/json").unwrap();
    headers.insert("Content-Type", type_header);

    let macaroon_header =
        HeaderValue::from_str(macaroon.expose_secret()).expect("error building macaroon header");
    headers.insert("Grpc-Metadata-macaroon", macaroon_header);

    headers
}
#[instrument(skip_all)]
fn build_macaroon(macaroon_path: String) -> Secret<String> {
    let contents = fs::read(macaroon_path).expect("failed to macaroon file");
    let hex_string = buffer_as_hex(contents);
    Secret::from(hex_string)
}

#[instrument(skip_all)]
fn build_tls(tls_path: String) -> Secret<String> {
    let contents = fs::read(tls_path).expect("failed to read tls file");
    let hex_string = buffer_as_hex(contents);
    Secret::from(hex_string)
}
fn buffer_as_hex(bytes: Vec<u8>) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

impl Future for LndManager {
    type Output = Result<(), std::io::Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let async_fn = async { self.clone().run().await };
        let mut future = Box::pin(async_fn);

        match future.as_mut().poll(cx) {
            Poll::Ready(result) => match result {
                Ok(_) => Poll::Ready(Ok(())),
                Err(e) => Poll::Ready(Err(std::io::Error::new(
                    ErrorKind::Other,
                    format!("unexpected error in running lnd manager: {:?}", e),
                ))),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
