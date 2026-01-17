use crate::orchestrator::Orchestrator;
use crate::web::api::{self, AppState};
use crate::Result;
use axum::body::Body;
use axum::http::{header, Response, StatusCode, Uri};
use axum::routing::{get, post};
use axum::Router;
use rust_embed::Embed;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

const DEFAULT_PORT: u16 = 3847;

#[derive(Embed)]
#[folder = "dashboard/dist"]
struct Assets;

async fn fallback_handler(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    serve_file(if path.is_empty() { "index.html" } else { path })
}

fn serve_file(path: &str) -> Response<Body> {
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // For SPA routing, serve index.html for unknown paths
            if let Some(content) = Assets::get("index.html") {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(content.data.into_owned()))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not Found"))
                    .unwrap()
            }
        }
    }
}

pub async fn run_server(port: Option<u16>, open_browser: bool) -> Result<()> {
    let port = port.unwrap_or(DEFAULT_PORT);
    let orchestrator = Orchestrator::new()?;
    let state: AppState = Arc::new(Mutex::new(orchestrator));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .route("/agents", get(api::list_agents))
        .route("/agents/:id", get(api::get_agent).delete(api::remove_agent))
        .route("/agents/:id/diff", get(api::get_diff))
        .route("/agents/:id/merge", post(api::merge_agent))
        .route("/agents/:id/pr", post(api::create_pr))
        .route("/agents/:id/output", get(api::get_output));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback(fallback_handler)
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://localhost:{port}");

    println!("Starting WTA Dashboard at {url}");

    if open_browser {
        let _ = open::that(&url);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
