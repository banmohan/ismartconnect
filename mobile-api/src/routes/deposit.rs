use axum::extract::{Extension, Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::auth::ClientIdentity;
use crate::cbs::{
    AccountValidationResponse, AllAccountResponse, BalanceResponse, CbsAccountValidation, CbsAllAccount, CbsBalance,
    CbsMiniStatement, FundTransferBody, FundTransferResponse, MiniStatementResponse, ReqAccount, ReqFullStatement,
    ReqFundTransfer, StatementListResponse,
};
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/deposit/balance", post(balance))
        .route("/api/deposit/mini-statement", post(mini_statement))
        .route("/api/deposit/full-statement", post(full_statement))
        .route("/api/deposit/fund-transfer", post(fund_transfer))
        .route("/api/deposit/reversal", post(reversal))
        .route("/api/deposit/acc-validation", post(acc_validation))
        .route("/api/deposit/all-account/{member_id}", get(all_account))
}

#[utoipa::path(
    post,
    path = "/api/deposit/balance",
    tag = "deposit",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = ReqAccount,
    responses(
        (status = 200, description = "Available and ledger balances", body = BalanceResponse),
        (status = 400, description = "API key and tenant do not match"),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn balance(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<ReqAccount>,
) -> Result<Json<BalanceResponse>, ApiError> {
    let cbs: CbsBalance = state.cbs.post("api/mbank/balance", &client.code, &body).await?;
    Ok(Json(BalanceResponse {
        available_balance: cbs.available_balance,
        ledger_balance: cbs.ledger_balance,
        iso_response_code: "00".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/deposit/mini-statement",
    tag = "deposit",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = ReqAccount,
    responses(
        (status = 200, description = "Mini statement", body = MiniStatementResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn mini_statement(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<ReqAccount>,
) -> Result<Json<MiniStatementResponse>, ApiError> {
    let cbs: CbsMiniStatement = state.cbs.post("api/mbank/mini-statement", &client.code, &body).await?;
    Ok(Json(MiniStatementResponse {
        available_balance: cbs.available_balance,
        statement_list: cbs.data,
        iso_response_code: "00".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/deposit/full-statement",
    tag = "deposit",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = ReqFullStatement,
    responses(
        (status = 200, description = "Full statement", body = StatementListResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn full_statement(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<ReqFullStatement>,
) -> Result<Json<StatementListResponse>, ApiError> {
    let statement_list = state.cbs.post("api/mbank/full-statement", &client.code, &body).await?;
    Ok(Json(StatementListResponse { statement_list, iso_response_code: "00".to_string() }))
}

#[utoipa::path(
    post,
    path = "/api/deposit/fund-transfer",
    tag = "deposit",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = ReqFundTransfer,
    responses(
        (status = 200, description = "Transfer accepted", body = FundTransferResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn fund_transfer(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<ReqFundTransfer>,
) -> Result<Json<FundTransferResponse>, ApiError> {
    let cbs: FundTransferBody = state.cbs.post("api/mbank/fund-transfer", &client.code, &body).await?;
    Ok(Json(FundTransferResponse { tran_code: cbs.tran_code, iso_response_code: "00".to_string() }))
}

#[utoipa::path(
    post,
    path = "/api/deposit/reversal",
    tag = "deposit",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = FundTransferBody,
    responses(
        (status = 200, description = "Reversal accepted", body = FundTransferResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn reversal(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<FundTransferBody>,
) -> Result<Json<FundTransferResponse>, ApiError> {
    let _accepted: bool = state.cbs.post("api/mbank/reversal", &client.code, &body).await?;
    Ok(Json(FundTransferResponse { tran_code: body.tran_code, iso_response_code: "00".to_string() }))
}

#[utoipa::path(
    post,
    path = "/api/deposit/acc-validation",
    tag = "deposit",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key")
    ),
    request_body = ReqAccount,
    responses(
        (status = 200, description = "Account validation", body = AccountValidationResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn acc_validation(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Json(body): Json<ReqAccount>,
) -> Result<Json<AccountValidationResponse>, ApiError> {
    let cbs: CbsAccountValidation = state.cbs.post("api/mbank/acc-validation", &client.code, &body).await?;
    Ok(Json(AccountValidationResponse::from_cbs(cbs)))
}

#[utoipa::path(
    get,
    path = "/api/deposit/all-account/{member_id}",
    tag = "deposit",
    params(
        ("tenant" = String, Header, description = "Client code matching the API key"),
        ("member_id" = i64, Path, description = "Member id")
    ),
    responses(
        (status = 200, description = "Deposit, loan, and share accounts", body = AllAccountResponse),
        (status = 401, description = "Unknown API key"),
        (status = 500, description = "CBS failure", body = crate::error::ErrorBody)
    )
)]
pub async fn all_account(
    State(state): State<AppState>,
    Extension(client): Extension<ClientIdentity>,
    Path(member_id): Path<i64>,
) -> Result<Json<AllAccountResponse>, ApiError> {
    let path = format!("api/mbank/all-account-ismart/{member_id}");
    let cbs: CbsAllAccount = state.cbs.get(&path, &client.code).await?;
    Ok(Json(AllAccountResponse {
        deposit: cbs.deposit,
        loan: cbs.loan,
        share: cbs.share,
        iso_response_code: "00".to_string(),
    }))
}
