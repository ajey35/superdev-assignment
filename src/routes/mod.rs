use axum::routing::post;
use axum::Router;
use crate::handlers::{keypair, token,message,send};

pub fn router() -> Router {
    Router::new()
        .route("/keypair", post(keypair::generate))
        .route("/token/create", post(token::create))
        .route("/token/mint", post(token::mint))
        .route("/message/sign", post(message::sign))
        .route("/message/verify", post(message::verify))
        .route("/send/sol", post(send::send_sol))
        .route("/send/token", post(send::send_token))
}
