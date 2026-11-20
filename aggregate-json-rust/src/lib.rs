use serde::{Deserialize, Serialize};
use spin_sdk::http::body::IncomingBodyExt;
use spin_sdk::http::{IntoResponse, Request, Response, StatusCode};
use spin_sdk::http_service;

/// The only field we need from SWAPI people and planets
#[derive(Deserialize)]
struct Named {
    name: String,
}

/// The aggregated response we send back
#[derive(Serialize)]
struct Aggregate {
    person: String,
    planet: String,
}

#[http_service]
async fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    if req.uri().path() != "/" {
        return json(StatusCode::NOT_FOUND, r#"{"error":"not found"}"#.to_string());
    }

    // Send both requests concurrently and wait until both have finished
    let (person, planet) = futures::join!(
        fetch_name("https://swapi.dev/api/people/1/"),
        fetch_name("https://swapi.dev/api/planets/1/"),
    );

    match (person, planet) {
        // Both succeeded: combine one field of each into a new JSON object
        (Ok(person), Ok(planet)) => {
            let body = serde_json::to_string(&Aggregate { person, planet })?;
            json(StatusCode::OK, body)
        }
        // At least one upstream request failed
        _ => json(
            StatusCode::BAD_GATEWAY,
            r#"{"error":"upstream request failed"}"#.to_string(),
        ),
    }
}

/// Fetches a SWAPI resource and returns its "name" field
async fn fetch_name(url: &str) -> anyhow::Result<String> {
    let response = spin_sdk::http::get(url).await?;
    anyhow::ensure!(response.status().is_success(), "upstream returned {}", response.status());
    let body = response.into_body().bytes().await?;
    Ok(serde_json::from_slice::<Named>(&body)?.name)
}

/// Builds a response with the given status and JSON body
fn json(status: StatusCode, body: String) -> anyhow::Result<Response<String>> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body)?)
}
