use futures::{SinkExt, StreamExt};
use lol_html::html_content::ContentType;
use lol_html::{HtmlRewriter, Settings, element};
use spin_sdk::http::body::{IncomingBodyExt, stream};
use spin_sdk::http::{EmptyBody, IntoResponse, Method, Request, Response};
use spin_sdk::wasip3;
use spin_sdk::{http_service, variables};
use std::cell::RefCell;
use std::rc::Rc;

#[http_service]
async fn handle_html_rewrite(_req: Request) -> anyhow::Result<impl IntoResponse> {
    // Load template.html from the "origin" (local service chaining for demo purposes)
    let origin = variables::get("origin").await?;
    let origin_request = Request::builder()
        .method(Method::GET)
        .uri(format!("{origin}/template.html"))
        .body(EmptyBody::new())?;

    let origin_response = spin_sdk::http::send(origin_request).await?;
    let mut origin_stream = origin_response.into_body().stream();

    // The response body is itself a stream: bytes produced by lol_html are
    // forwarded to the client as soon as they're available.
    let (mut tx, response_body) = stream::<Vec<u8>>();

    wasip3::spawn(async move {
        // Buffers the bytes lol_html emits for the chunk currently being
        // processed. Shared with the rewriter's output sink via `Rc<RefCell<_>>`
        // since the sink closure must be callable synchronously from within
        // `rewriter.write`.
        let pending = Rc::new(RefCell::new(Vec::new()));
        let sink = pending.clone();

        // Rewrites HTML on the fly: prefixes the content of every <li>
        // element with "Iced ".
        let mut rewriter = HtmlRewriter::new(
            Settings::new().append_element_content_handler(element!("li", |el| {
                el.prepend("Iced ", ContentType::Text);
                Ok(())
            })),
            move |chunk: &[u8]| sink.borrow_mut().extend_from_slice(chunk),
        );

        while let Some(chunk) = origin_stream.next().await {
            let chunk = match chunk {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("error reading origin body: {e}");
                    break;
                }
            };

            if let Err(e) = rewriter.write(&chunk) {
                eprintln!("error rewriting html: {e}");
                break;
            }

            let flushed = {
                let mut buf = pending.borrow_mut();
                (!buf.is_empty()).then(|| std::mem::take(&mut *buf))
            };

            if let Some(data) = flushed {
                if tx.send(data).await.is_err() {
                    // Client disconnected; stop pulling from the origin.
                    return;
                }
            }
        }

        if let Err(e) = rewriter.end() {
            eprintln!("error finalizing html rewrite: {e}");
        }

        let remaining = std::mem::take(&mut *pending.borrow_mut());
        if !remaining.is_empty() {
            let _ = tx.send(remaining).await;
        }
    });

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/html; charset=utf-8")
        .body(response_body)?)
}
