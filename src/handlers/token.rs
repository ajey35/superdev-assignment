use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::Instruction,
    pubkey::Pubkey,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    id as token_program_id,
};
use crate::models::response::ApiResponse;
use base64::{engine::general_purpose, Engine as _};
use std::str::FromStr;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenRequest {
    pub mint_authority: String,
    pub mint: String,
    pub decimals: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintTokenRequest {
    pub mint: String,
    pub destination: String,
    pub authority: String,
    pub amount: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInstructionData {
    pub program_id: String,
    pub accounts: Vec<AccountInfo>,
    pub instruction_data: String,
}

pub async fn create(Json(req): Json<CreateTokenRequest>) -> impl IntoResponse {
    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(k) => k,
        Err(_) => return error("Invalid mint pubkey"),
    };
    let authority = match Pubkey::from_str(&req.mint_authority) {
        Ok(k) => k,
        Err(_) => return error("Invalid mint authority pubkey"),
    };

    let instruction = match initialize_mint(
        &token_program_id(),
        &mint_pubkey,
        &authority,
        None,
        req.decimals,
    ) {
        Ok(ix) => ix,
        Err(_) => return error("Failed to create initialize_mint instruction"),
    };

    respond(instruction)
}

pub async fn mint(Json(req): Json<MintTokenRequest>) -> impl IntoResponse {
    let mint = match Pubkey::from_str(&req.mint) {
        Ok(k) => k,
        Err(_) => return error("Invalid mint pubkey"),
    };
    let destination = match Pubkey::from_str(&req.destination) {
        Ok(k) => k,
        Err(_) => return error("Invalid destination pubkey"),
    };
    let authority = match Pubkey::from_str(&req.authority) {
        Ok(k) => k,
        Err(_) => return error("Invalid authority pubkey"),
    };

    let instruction = match mint_to(
        &token_program_id(),
        &mint,
        &destination,
        &authority,
        &[],
        req.amount,
    ) {
        Ok(ix) => ix,
        Err(_) => return error("Failed to create mint_to instruction"),
    };

    respond(instruction)
}

fn respond(inst: Instruction) -> (StatusCode, Json<ApiResponse<TokenInstructionData>>) {
    let accounts = inst.accounts.iter().map(|acc| AccountInfo {
        pubkey: acc.pubkey.to_string(),
        is_signer: acc.is_signer,
        is_writable: acc.is_writable,
    }).collect();

    let encoded = general_purpose::STANDARD.encode(&inst.data);
    let response = TokenInstructionData {
        program_id: inst.program_id.to_string(),
        accounts,
        instruction_data: encoded,
    };

    (StatusCode::OK, Json(ApiResponse::ok(response)))
}

fn error<T>(msg: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::BAD_REQUEST, Json(ApiResponse::err(msg)))
}
