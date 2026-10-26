use axum::extract::{Extension, State};
use axum::routing::post;
use axum::{Json, Router};

use crate::auth::ClientIdentity;
use crate::cbs::{
    CbsLoanDetails, LoanDetailResponse, LoanScheduleItem, LoanScheduleResponse, LoanStatementListResponse, ReqAccount,
    ReqLoanDetail, ReqLoanStatement,
};
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/loan/loan-detail", post(loan_detail))
        .route("/api/loan/loan-statement", post(loan_statement))
        .route("/api/loan/loan-schedule", post(loan_schedule))
}

#[utoipa::path(
    post,
    path = "/api/loan/loan-detail",
    tag = "loan",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = ReqLoanDetail,
    responses(
        (status = 200, description = "Loan detail", body = LoanDetailResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn loan_detail(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<ReqLoanDetail>,
) -> Result<Json<LoanDetailResponse>, ApiError> {
    let cbs: CbsLoanDetails = state.cbs.post("api/mbank/loan-detail", &client.code, &body).await?;
    Ok(Json(LoanDetailResponse::from_cbs(cbs)))
}

#[utoipa::path(
    post,
    path = "/api/loan/loan-statement",
    tag = "loan",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = ReqLoanStatement,
    responses(
        (status = 200, description = "Loan statement", body = LoanStatementListResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn loan_statement(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<ReqLoanStatement>,
) -> Result<Json<LoanStatementListResponse>, ApiError> {
    let statement_list = state.cbs.post("api/mbank/loan-statement", &client.code, &body).await?;
    Ok(Json(LoanStatementListResponse { statement_list, iso_response_code: "00".to_string() }))
}

#[utoipa::path(
    post,
    path = "/api/loan/loan-schedule",
    tag = "loan",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = ReqAccount,
    responses(
        (status = 200, description = "Loan schedule", body = LoanScheduleResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn loan_schedule(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<ReqAccount>,
) -> Result<Json<LoanScheduleResponse>, ApiError> {
    let schedules: Vec<LoanScheduleItem> = state.cbs.post("api/mbank/loan-schedule", &client.code, &body).await?;
    Ok(Json(LoanScheduleResponse { schedules, iso_response_code: "00".to_string() }))
}
