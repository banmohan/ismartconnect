use std::net::SocketAddr;
use std::time::Instant;

use axum::body::Body;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use chrono::Utc;
use uuid::Uuid;

use crate::cbs::json_message;
use crate::state::AppState;
use crate::text::{truncate, COLLECT_MAX, LOG_BODY_MAX};

use super::extract::{build_search_text, extract_fields};
use super::writer::LogRecord;
use super::{new_slot, snapshot, LOG_SLOT};

pub async fn log_api(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    if !path.starts_with("/api/") {
        return next.run(req).await;
    }

    let request_id = Uuid::new_v4();
    let requested_at = Utc::now();
    let method = req.method().to_string();
    let query_string = req.uri().query().map(|query| truncate(query, 2000));
    let request_headers = truncate(&format_headers(req.headers()), LOG_BODY_MAX);
    let ip_address = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string());

    let (req, request_body_full) = match buffer_request(req).await {
        Ok(prepared) => prepared,
        Err(response) => return with_request_id(response, request_id),
    };
    let request_body = truncate(&request_body_full, LOG_BODY_MAX);

    let slot = new_slot();
    let started = Instant::now();
    let response = LOG_SLOT.scope(slot.clone(), next.run(req)).await;
    let duration_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;
    let (mut response, response_body_full) = buffer_response(response).await;
    let response_body = truncate(&response_body_full, LOG_BODY_MAX);
    if let Ok(value) = request_id.to_string().parse() {
        response.headers_mut().insert("x-request-id", value);
    }

    let slot = snapshot(&slot);
    let extracted = extract_fields(&path, &request_body_full, &response_body_full);
    let status_code = response.status().as_u16() as i32;
    let error_message = slot.error_message.clone().or_else(|| {
        if status_code >= 400 {
            json_message(&response_body_full)
        } else {
            None
        }
    });
    let is_error = status_code >= 400 || error_message.is_some();
    let record = LogRecord {
        id: request_id,
        request_id,
        requested_at,
        duration_ms,
        http_method: method,
        path: truncate(&path, 500),
        query_string,
        client_id: slot.client_id,
        client_code: slot.client_code.clone(),
        ip_address,
        request_headers: Some(request_headers),
        request_body: Some(request_body),
        status_code,
        response_body: Some(response_body),
        is_error,
        error_message: error_message.as_ref().map(|message| truncate(message, 2000)),
        exception_details: None,
        iso_response_code: extracted.iso_response_code.clone(),
        account_number: extracted.account_number.clone(),
        src_account: extracted.src_account.clone(),
        dest_account: extracted.dest_account.clone(),
        member_id: extracted.member_id.clone(),
        tran_code: extracted.tran_code.clone(),
        amount: extracted.amount,
        upstream_path: slot.upstream_path,
        upstream_status: slot.upstream_status,
        upstream_duration_ms: slot.upstream_duration_ms,
        upstream_body: slot.upstream_body,
        search_text: build_search_text(&path, error_message.as_deref(), slot.client_code.as_deref(), &extracted),
    };
    state.logs.try_send(record);
    response
}

async fn buffer_request(req: Request) -> Result<(Request, String), Response> {
    let (parts, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, COLLECT_MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, "request body is too large").into_response());
        }
    };
    let text = String::from_utf8_lossy(&bytes).to_string();
    Ok((Request::from_parts(parts, Body::from(bytes)), text))
}

async fn buffer_response(response: Response) -> (Response, String) {
    let (parts, body) = response.into_parts();
    match axum::body::to_bytes(body, COLLECT_MAX).await {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes).to_string();
            (Response::from_parts(parts, Body::from(bytes)), text)
        }
        Err(_) => {
            let text = "…[truncated]".to_string();
            (Response::from_parts(parts, Body::from(text.clone())), text)
        }
    }
}

fn with_request_id(mut response: Response, request_id: Uuid) -> Response {
    if let Ok(value) = request_id.to_string().parse() {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn format_headers(headers: &HeaderMap) -> String {
    let mut out = String::new();
    for (name, value) in headers {
        let rendered = if name.as_str().eq_ignore_ascii_case(header::AUTHORIZATION.as_str())
            || name.as_str().eq_ignore_ascii_case("x-api-key")
        {
            "[REDACTED]"
        } else {
            value.to_str().unwrap_or("[binary]")
        };
        out.push_str(name.as_str());
        out.push_str(": ");
        out.push_str(rendered);
        out.push('\n');
    }
    out
}

use axum::response::IntoResponse;
