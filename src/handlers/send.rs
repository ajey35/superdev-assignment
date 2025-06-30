use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use solana_program::{system_instruction, pubkey::Pubkey};
use spl_token::instruction::transfer_checked;
use spl_token::id as token_program_id;
use base64::{engine::general_purpose, Engine as _};
use std::str::FromStr;
use crate::models::response::ApiResponse;

// Note: system_instruction is deprecated, but kept for compatibility. Consider updating to solana_system_interface in the future.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendSolRequest {
    pub from: String,
    pub to: String,
    pub lamports: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTokenRequest {
    pub destination: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolInstructionData {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub instruction_data: String,
}

#[derive(Serialize)]
pub struct TokenAccountMeta {
    pub pubkey: String,
    #[serde(rename = "isSigner")]
    pub is_signer: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInstructionData {
    pub program_id: String,
    pub accounts: Vec<TokenAccountMeta>,
    pub instruction_data: String,
}

pub async fn send_sol(Json(req): Json<SendSolRequest>) -> impl IntoResponse {
    let from = match Pubkey::from_str(&req.from) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid sender address"),
    };
    let to = match Pubkey::from_str(&req.to) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid recipient address"),
    };

    let ix = system_instruction::transfer(&from, &to, req.lamports);
    let encoded_data = general_purpose::STANDARD.encode(ix.data);

    let accounts = ix.accounts.iter().map(|a| a.pubkey.to_string()).collect();

    let payload = SolInstructionData {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: encoded_data,
    };

    (StatusCode::OK, Json(ApiResponse::ok(payload)))
}

pub async fn send_token(Json(req): Json<SendTokenRequest>) -> impl IntoResponse {
    let mint = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid mint address"),
    };
    let dest = match Pubkey::from_str(&req.destination) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid destination address"),
    };
    let owner = match Pubkey::from_str(&req.owner) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid owner address"),
    };

    let decimals = 6; // Assumption; should be validated/queried in real apps

    let ix = match transfer_checked(
        &token_program_id(),
        &owner,      // source token account
        &mint,
        &dest,       // destination token account
        &owner,
        &[],
        req.amount,
        decimals,
    ) {
        Ok(i) => i,
        Err(_) => return error("Failed to create SPL token transfer instruction"),
    };

    let encoded_data = general_purpose::STANDARD.encode(&ix.data);

    let accounts = ix.accounts.iter().map(|m| TokenAccountMeta {
        pubkey: m.pubkey.to_string(),
        is_signer: m.is_signer,
    }).collect();

    let payload = TokenInstructionData {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: encoded_data,
    };

    (StatusCode::OK, Json(ApiResponse::ok(payload)))
}

fn error<T>(msg: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::BAD_REQUEST, Json(ApiResponse::err(msg)))
}
