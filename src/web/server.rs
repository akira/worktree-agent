use crate::orchestrator::Orchestrator;
use crate::web::api::{self, AppState};
use crate::Result;
use axum::routing::{delete, get, post};
use axum::Router;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

const DEFAULT_PORT: u16 = 3847;

pub async fn run_server(port: Option<u16>, open_browser: bool) -> Result<()> {
    let port = port.unwrap_or(DEFAULT_PORT);
    let orchestrator = Orchestrator::new()?;
    let state: AppState = Arc::new(Mutex::new(orchestrator));

    // Determine static files directory
    let static_dir = find_static_dir();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .route("/agents", get(api::list_agents))
        .route("/agents/{id}", get(api::get_agent))
        .route("/agents/{id}/diff", get(api::get_diff))
        .route("/agents/{id}/merge", post(api::merge_agent))
        .route("/agents/{id}/pr", post(api::create_pr))
        .route("/agents/{id}/output", get(api::get_output))
        .route("/agents/{id}", delete(api::remove_agent));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new(&static_dir).append_index_html_on_directories(true))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://localhost:{port}");

    println!("Starting WTA Dashboard at {url}");
    println!("Static files served from: {}", static_dir.display());

    if open_browser {
        let _ = open::that(&url);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn find_static_dir() -> PathBuf {
    // Try to find the dashboard directory relative to the executable or current directory
    let candidates = [
        // Development: relative to crate root
        PathBuf::from("dashboard/dist"),
        // Installed: relative to executable
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("dashboard")))
            .unwrap_or_default(),
        // Fallback: current directory
        PathBuf::from("./dashboard/dist"),
    ];

    for candidate in &candidates {
        if candidate.exists() && candidate.is_dir() {
            return candidate.clone();
        }
    }

    // Default to dashboard/dist even if it doesn't exist yet
    PathBuf::from("dashboard/dist")
}
