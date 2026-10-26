use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::logging;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{message}")]
    Mapped { message: String, iso: &'static str },
    #[error("{0}")]
    Transport(String),
}

impl ApiError {
    pub fn mapped(message: impl Into<String>, iso: &'static str) -> Self {
        Self::Mapped { message: message.into(), iso }
    }

    pub fn transport(err: impl std::fmt::Display) -> Self {
        Self::Transport(err.to_string())
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub message: String,
    pub error_code: u16,
    pub iso_response_code: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (message, iso) = match self {
            ApiError::Mapped { message, iso } => (message, iso),
            ApiError::Transport(message) => (message, "96"),
        };
        logging::set_error(&message);
        let body = ErrorBody {
            message,
            error_code: 500,
            iso_response_code: iso.to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}
