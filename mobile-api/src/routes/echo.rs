use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::ClientIdentity;
use crate::cbs::EchoResponse;
use crate::logging;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/echo/echo", get(echo))
}

#[utoipa::path(
    get,
    path = "/api/echo/echo",
    tag = "echo",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    responses(
        (status = 200, description = "CBS echo succeeded", body = EchoResponse),
        (status = 400, description = "API key and tenant do not match"),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS echo failed", body = crate::error::ErrorBody)
    )
)]
pub async fn echo(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
) -> impl IntoResponse {
    match state.cbs.get::<bool>("api/mbank/echo", &client.code).await {
        Ok(true) => Json(EchoResponse { result: "success".to_string() }).into_response(),
        Ok(false) => {
            logging::set_error("echo returned false");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(err) => err.into_response(),
    }
}
