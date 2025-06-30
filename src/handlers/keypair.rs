
use axum::{http::StatusCode, response::IntoResponse};
use axum::Json;
use solana_sdk::{signature::Keypair, signer::Signer};
use serde::Serialize;
use crate::models::response::ApiResponse;
use bs58;

#[derive(Serialize)]
struct KeypairData {
    pubkey: String,
    secret: String,
}

pub async fn generate() -> impl IntoResponse {
    let kp = Keypair::new();
    let data = KeypairData {
        pubkey: kp.pubkey().to_string(),
        secret: bs58::encode(kp.to_bytes()).into_string(),
    };
    let body = ApiResponse::ok(data);
    (StatusCode::OK, Json(body))
}
