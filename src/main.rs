
use std::{str::FromStr, sync::Arc};

use axum::{extract::State, routing::post, Json};
use bdk_wallet::{SignOptions, Wallet};
use bitcoin::Psbt;
use serde::{Deserialize, Serialize, Serializer};

pub struct AppState {
    pub wallet: Wallet,
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub port: u16,
    pub network: bitcoin::Network,
    pub xprv: String,
}

impl AppState {
    pub async fn init(config: &Config) -> Result<Self, String> {

        let wallet = bdk_wallet::Wallet::create_single(config.xprv.clone())
            .network(config.network)
            .create_wallet_no_persist()
            .expect("create wallet");

        let app = AppState { wallet };

        Ok(app)
    }
}

#[tokio::main]
async fn main() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_timer(
            tracing_subscriber::fmt::time::ChronoLocal::new("%FT%H:%M:%S%z".to_owned()),
        ))
        .init();
    let config_path = std::env::args().nth(1).expect("config path");
    let config = std::fs::read_to_string(config_path).unwrap();
    let config: Config = toml::from_str(&config).unwrap();

    run(config).await;
}


async fn health() -> &'static str {
    "ok"
}

use axum::routing::get;

async fn run(config: Config) {
    let state = AppState::init(&config).await.unwrap();
    let listen = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();
    let state = Arc::new(state);
    let router = axum::Router::new()
        .route("/sign_psbt", post(sign_service))
        .route("/health", get(health))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    tracing::info!("listen on: {}", listen.local_addr().unwrap());
    axum::serve(listen, router).await.unwrap();
}

async fn sign_service(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignRequest>,
) -> Result<Json<SignResponse>, Error> {
    let mut signed_psbt = req.psbt;

    let sign_options = SignOptions {
        trust_witness_utxo: true,
        allow_all_sighashes: true,
        ..Default::default()
    };
    state
        .wallet
        .sign(&mut signed_psbt, sign_options)
        .map_err(|e| Error::InvalidTransaction(format!("signing failed: {e}")))?;

    Ok(Json(SignResponse { psbt: signed_psbt }))
}

#[derive(serde::Deserialize)]
pub struct SignRequest {
    #[serde(deserialize_with = "de_psbt_from_base64")]
    pub psbt: Psbt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignResponse {
    #[serde(serialize_with = "serialize_psbt_to_base64")]
    pub psbt: Psbt,
}

pub fn de_psbt_from_base64<'de, D>(deserializer: D) -> Result<Psbt, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    Psbt::from_str(s).map_err(serde::de::Error::custom)
}

pub fn serialize_psbt_to_base64<S>(psbt: &Psbt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let base64 = psbt.to_string(); 
    serializer.serialize_str(&base64)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(self = ?self, "error");
        use axum::http::StatusCode;
        use Error::*;
        match self {
            InvalidTransaction(e) => (StatusCode::BAD_REQUEST, e),
        }
        .into_response()
    }
}
