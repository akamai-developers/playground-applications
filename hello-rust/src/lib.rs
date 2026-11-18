use serde::{Deserialize, Serialize};
use spin_sdk::http::body::IncomingBodyExt;
use spin_sdk::http::{IntoResponse, Json, Request};
use spin_sdk::http_service;
#[derive(Deserialize)]
pub struct Payload {
    pub name: String,
}

#[derive(Serialize)]
pub struct ResponsePayload {
    pub message: String,
}
#[http_service]
async fn handle_hello_rust(req: Request) -> anyhow::Result<impl IntoResponse> {
    let body = req.into_body().bytes().await?;
    let payload = serde_json::from_slice::<Payload>(&body)?;
    let res_payload = ResponsePayload {
        message: format!("Hello, {}", payload.name),
    };
    Ok(Json(res_payload))
}
