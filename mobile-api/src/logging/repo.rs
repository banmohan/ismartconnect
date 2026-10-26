use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use super::query::{build_clauses, Clause, LogFilter};

const SUMMARY_COLUMNS: &str = "id, request_id, requested_at, duration_ms, http_method, path, client_code, status_code, is_error, iso_response_code, account_number, src_account, dest_account, member_id, tran_code";

#[derive(Clone, Debug)]
pub struct LogSummary {
    pub id: Uuid,
    pub request_id: Uuid,
    pub requested_at: DateTime<Utc>,
    pub duration_ms: i32,
    pub http_method: String,
    pub path: String,
    pub client_code: Option<String>,
    pub status_code: i32,
    pub is_error: bool,
    pub iso_response_code: Option<String>,
    pub account_number: Option<String>,
    pub src_account: Option<String>,
    pub dest_account: Option<String>,
    pub member_id: Option<String>,
    pub tran_code: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LogRow {
    pub id: Uuid,
    pub request_id: Uuid,
    pub requested_at: DateTime<Utc>,
    pub duration_ms: i32,
    pub http_method: String,
    pub path: String,
    pub query_string: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_code: Option<String>,
    pub ip_address: Option<String>,
    pub request_headers: Option<String>,
    pub request_body: Option<String>,
    pub status_code: i32,
    pub response_body: Option<String>,
    pub is_error: bool,
    pub error_message: Option<String>,
    pub exception_details: Option<String>,
    pub iso_response_code: Option<String>,
    pub account_number: Option<String>,
    pub src_account: Option<String>,
    pub dest_account: Option<String>,
    pub member_id: Option<String>,
    pub tran_code: Option<String>,
    pub amount: Option<Decimal>,
    pub upstream_path: Option<String>,
    pub upstream_status: Option<i32>,
    pub upstream_duration_ms: Option<i32>,
    pub upstream_body: Option<String>,
    pub search_text: Option<String>,
}

pub struct LogPage {
    pub rows: Vec<LogSummary>,
    pub total: i64,
}

pub async fn search_logs(pool: &PgPool, filter: &LogFilter) -> Result<LogPage, sqlx::Error> {
    let now = Utc::now();
    let clauses = build_clauses(filter, now);
    let total = count_logs(pool, &clauses).await?;
    let mut qb = QueryBuilder::new(format!("SELECT {SUMMARY_COLUMNS} FROM request_logs WHERE 1=1"));
    push_clauses(&mut qb, &clauses);
    qb.push(" ORDER BY ");
    qb.push(filter.order_sql());
    qb.push(" LIMIT ");
    qb.push_bind(filter.page_size);
    qb.push(" OFFSET ");
    qb.push_bind((filter.page - 1) * filter.page_size);
    let rows = qb.build().map(summary_from_row).fetch_all(pool).await?;
    Ok(LogPage { rows, total })
}

pub async fn log_by_id(pool: &PgPool, id: Uuid) -> Result<Option<LogRow>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, request_id, requested_at, duration_ms, http_method, path, query_string, client_id, client_code,
                ip_address, request_headers, request_body, status_code, response_body, is_error, error_message,
                exception_details, iso_response_code, account_number, src_account, dest_account, member_id,
                tran_code, amount, upstream_path, upstream_status, upstream_duration_ms, upstream_body, search_text
         FROM request_logs WHERE id = $1 ORDER BY requested_at DESC LIMIT 1",
    )
    .bind(id)
    .map(row_from_pg)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn related_logs(pool: &PgPool, row: &LogRow) -> Result<Vec<LogSummary>, sqlx::Error> {
    if row.tran_code.is_none() && row.account_number.is_none() && row.src_account.is_none() && row.dest_account.is_none() {
        return Ok(Vec::new());
    }
    let account = row.account_number.clone().or_else(|| row.src_account.clone()).or_else(|| row.dest_account.clone());
    let from = row.requested_at - Duration::minutes(30);
    let to = row.requested_at + Duration::minutes(30);
    let rows = sqlx::query(&format!(
        "SELECT {SUMMARY_COLUMNS} FROM request_logs
         WHERE id <> $1 AND requested_at BETWEEN $2 AND $3
           AND (($4::text IS NOT NULL AND tran_code = $4)
             OR ($5::text IS NOT NULL AND (account_number = $5 OR src_account = $5 OR dest_account = $5)))
         ORDER BY requested_at DESC LIMIT 20"
    ))
    .bind(row.id)
    .bind(from)
    .bind(to)
    .bind(&row.tran_code)
    .bind(&account)
    .map(summary_from_row)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn count_logs(pool: &PgPool, clauses: &[Clause]) -> Result<i64, sqlx::Error> {
    let mut qb = QueryBuilder::new("SELECT COUNT(*) FROM request_logs WHERE 1=1");
    push_clauses(&mut qb, clauses);
    let count: i64 = qb.build_query_scalar().fetch_one(pool).await?;
    Ok(count)
}

fn push_clauses<'a>(qb: &mut QueryBuilder<'a, sqlx::Postgres>, clauses: &'a [Clause]) {
    for clause in clauses {
        match clause {
            Clause::Raw(sql) => {
                qb.push(*sql);
            }
            Clause::Text(value) => {
                qb.push_bind(value);
            }
            Clause::Int(value) => {
                qb.push_bind(*value);
            }
            Clause::Bool(value) => {
                qb.push_bind(*value);
            }
            Clause::Time(value) => {
                qb.push_bind(*value);
            }
        }
    }
}

fn summary_from_row(row: PgRow) -> LogSummary {
    LogSummary {
        id: row.get("id"),
        request_id: row.get("request_id"),
        requested_at: row.get("requested_at"),
        duration_ms: row.get("duration_ms"),
        http_method: row.get("http_method"),
        path: row.get("path"),
        client_code: row.get("client_code"),
        status_code: row.get("status_code"),
        is_error: row.get("is_error"),
        iso_response_code: row.get("iso_response_code"),
        account_number: row.get("account_number"),
        src_account: row.get("src_account"),
        dest_account: row.get("dest_account"),
        member_id: row.get("member_id"),
        tran_code: row.get("tran_code"),
    }
}

fn row_from_pg(row: PgRow) -> LogRow {
    LogRow {
        id: row.get("id"),
        request_id: row.get("request_id"),
        requested_at: row.get("requested_at"),
        duration_ms: row.get("duration_ms"),
        http_method: row.get("http_method"),
        path: row.get("path"),
        query_string: row.get("query_string"),
        client_id: row.get("client_id"),
        client_code: row.get("client_code"),
        ip_address: row.get("ip_address"),
        request_headers: row.get("request_headers"),
        request_body: row.get("request_body"),
        status_code: row.get("status_code"),
        response_body: row.get("response_body"),
        is_error: row.get("is_error"),
        error_message: row.get("error_message"),
        exception_details: row.get("exception_details"),
        iso_response_code: row.get("iso_response_code"),
        account_number: row.get("account_number"),
        src_account: row.get("src_account"),
        dest_account: row.get("dest_account"),
        member_id: row.get("member_id"),
        tran_code: row.get("tran_code"),
        amount: row.get("amount"),
        upstream_path: row.get("upstream_path"),
        upstream_status: row.get("upstream_status"),
        upstream_duration_ms: row.get("upstream_duration_ms"),
        upstream_body: row.get("upstream_body"),
        search_text: row.get("search_text"),
    }
}
