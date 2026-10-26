#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::time::Duration;

use axum::http::StatusCode;
use axum::middleware;
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::require_api_key;
use crate::logging::log_api;
use crate::openapi::ApiDoc;
use utoipa::OpenApi;

pub mod auth;
pub mod cbs;
mod config;
mod db;
mod error;
pub mod iso;
pub mod logging;
mod openapi;
mod routes;
mod serde_util;
pub mod state;
mod text;

pub use auth::KeyStore;
pub use cbs::CbsClient;
pub use state::AppState;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .merge(routes::echo::router())
        .merge(routes::deposit::router())
        .merge(routes::loan::router())
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(90)))
        .layer(CatchPanicLayer::new())
        .layer(middleware::from_fn_with_state(state.clone(), require_api_key))
        .layer(middleware::from_fn_with_state(state.clone(), log_api));

    Router::new()
        .merge(api)
        .nest("/admin", routes::admin::router(state.clone()))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .route("/static/app.css", get(asset_css))
        .route("/static/htmx.min.js", get(asset_htmx))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn root() -> Redirect {
    Redirect::temporary("/admin")
}

async fn asset_css() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")], include_str!("../static/app.css"))
}

async fn asset_htmx() -> impl axum::response::IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../static/htmx.min.js"),
    )
}

pub async fn run() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,sqlx=warn")),
        )
        .init();

    let config = config::Config::from_env()?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    db::migrate(&pool).await?;
    db::ensure_log_partitions(&pool).await?;
    db::seed_admin(&pool, &config).await?;
    let keys = db::load_keys(&pool).await?;
    let logs = logging::LogWorker::spawn(pool.clone());
    let cbs = CbsClient::new(&config.cbs_account_url, &config.cbs_api_key, config.cbs_timeout_secs)?;
    let state = AppState::new(pool, cbs, keys, logs.bus(), &config.cookie_secret, config.cookie_secure);
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "MobileApi listening");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    logs.shutdown().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install terminate handler");
        tokio::select! {
            _ = ctrl_c => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = ctrl_c.await;
    }
}
