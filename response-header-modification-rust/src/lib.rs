use spin_sdk::http::body::IncomingBodyExt;
use spin_sdk::http::{EmptyBody, FullBody, IntoResponse, Method, Request, Response, StatusCode};
use spin_sdk::{http_service, variables};

#[http_service]
async fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    // This sample only proxies GET requests
    if req.method() != Method::GET {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(FullBody::new("method not allowed".into()))?);
    }

    // The origin is configured through the Spin variable "origin"
    let origin = variables::get("origin").await?;

    // Build the origin URL: origin + incoming path + query string
    let path_and_query = req.uri().path_and_query().map_or("/", |pq| pq.as_str());
    let url = format!("{origin}{path_and_query}");

    // Forward the incoming accept header (or */* if the client didn't send one)
    let accept = req
        .headers()
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*/*");

    let origin_request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .header("accept", accept)
        .body(EmptyBody::new())?;
    let origin_response = spin_sdk::http::send(origin_request).await?;

    // Keep the origin status and content-type, drop all other origin headers
    let status = origin_response.status();
    let content_type = origin_response.headers().get("content-type").cloned();
    let body = origin_response.into_body().bytes().await?;

    // Build the response and add our own header
    let mut response = Response::builder()
        .status(status)
        .header("x-intercepted-value", "foobar");
    if let Some(content_type) = content_type {
        response = response.header("content-type", content_type);
    }
    Ok(response.body(FullBody::new(body))?)
}
