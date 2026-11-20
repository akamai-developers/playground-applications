use spin_sdk::http::body::IncomingBodyExt;
use spin_sdk::http::{IntoResponse, Request, Response, StatusCode};
use spin_sdk::http_service;

#[http_service]
async fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Route the request: "/" shows person 1, "/people/<id>" shows any person
    let path = req.uri().path();
    let id = match path.strip_prefix("/people/") {
        _ if path == "/" => "1",
        Some(id) if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) => id,
        Some(_) => return error(StatusCode::BAD_REQUEST, "id must be a number"),
        None => return error(StatusCode::NOT_FOUND, "not found"),
    };

    // Fetch the person from SWAPI (note the trailing slash, SWAPI redirects without it)
    let upstream = spin_sdk::http::get(format!("https://swapi.dev/api/people/{id}/")).await?;
    let status = upstream.status();
    let body = upstream.into_body().bytes().await?;

    // Return the upstream JSON unchanged, together with the upstream status
    json(status, String::from_utf8(body.to_vec())?)
}

/// Builds a JSON error response such as {"error":"not found"}
fn error(status: StatusCode, message: &str) -> anyhow::Result<Response<String>> {
    json(status, serde_json::json!({ "error": message }).to_string())
}

/// Builds a response with the given status and JSON body
fn json(status: StatusCode, body: String) -> anyhow::Result<Response<String>> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body)?)
}
