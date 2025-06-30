use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, SecretKey, PublicKey, Signer, Signature,SIGNATURE_LENGTH, KEYPAIR_LENGTH, SECRET_KEY_LENGTH,PUBLIC_KEY_LENGTH};
use base64::{engine::general_purpose, Engine as _};
use bs58;
use crate::models::response::ApiResponse;
use ed25519_dalek::Verifier;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignMessageRequest {
    pub message: String,
    pub secret: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignMessageResponse {
    pub signature: String,
    pub public_key: String,
    pub message: String,
}

pub async fn sign(Json(req): Json<SignMessageRequest>) -> impl IntoResponse {
    if req.message.trim().is_empty() || req.secret.trim().is_empty() {
        return error("Missing required fields");
    }

    let secret_bytes = match bs58::decode(&req.secret).into_vec() {
        Ok(b) if b.len() == SECRET_KEY_LENGTH || b.len() == KEYPAIR_LENGTH => b,
        _ => return error("Invalid base58-encoded secret"),
    };

    let keypair = if secret_bytes.len() == KEYPAIR_LENGTH {
        match Keypair::from_bytes(&secret_bytes) {
            Ok(kp) => kp,
            Err(_) => return error("Failed to parse keypair"),
        }
    } else {
        // Only secret key is given (32 bytes), derive public key
        match SecretKey::from_bytes(&secret_bytes) {
            Ok(secret) => {
                let public = PublicKey::from(&secret);
                Keypair { secret, public }
            }
            Err(_) => return error("Invalid secret key"),
        }
    };

    let signature = keypair.sign(req.message.as_bytes());
    let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());

    let response = SignMessageResponse {
        signature: signature_b64,
        public_key: bs58::encode(keypair.public.to_bytes()).into_string(),
        message: req.message,
    };

    (StatusCode::OK, Json(ApiResponse::ok(response)))
}

fn error<T>(msg: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::BAD_REQUEST, Json(ApiResponse::err(msg)))
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyMessageRequest {
    pub message: String,
    pub signature: String,
    pub pubkey: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyMessageResponse {
    pub valid: bool,
    pub message: String,
    pub pubkey: String,
}

pub async fn verify(Json(req): Json<VerifyMessageRequest>) -> impl IntoResponse {
    let sig_bytes = match general_purpose::STANDARD.decode(&req.signature) {
        Ok(bytes) if bytes.len() == SIGNATURE_LENGTH => bytes,
        _ => return error("Invalid base64 signature"),
    };

    let signature = match Signature::from_bytes(&sig_bytes) {
        Ok(sig) => sig,
        Err(_) => return error("Invalid signature format"),
    };

    let pubkey_bytes = match bs58::decode(&req.pubkey).into_vec() {
        Ok(bytes) if bytes.len() == PUBLIC_KEY_LENGTH => bytes,
        _ => return error("Invalid public key format"),
    };

    let public_key = match PublicKey::from_bytes(&pubkey_bytes) {
        Ok(pk) => pk,
        Err(_) => return error("Failed to parse public key"),
    };

    let is_valid = public_key.verify(req.message.as_bytes(), &signature).is_ok();

    let response = VerifyMessageResponse {
        valid: is_valid,
        message: req.message,
        pubkey: req.pubkey,
    };

    (StatusCode::OK, Json(ApiResponse::ok(response)))
}
