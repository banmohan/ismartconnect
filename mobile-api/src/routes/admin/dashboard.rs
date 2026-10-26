use std::collections::HashMap;

use askama::Template;
use axum::extract::{Extension, Query, State};
use axum::response::Response;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::state::AppState;

use super::{db_err, html, Shell};

#[derive(Deserialize, Default)]
pub struct DashQuery {
    notice: Option<String>,
    client_page: Option<i64>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage {
    shell: Shell,
    today: String,
    requests: String,
    failed: String,
    success: String,
    avg_ms: String,
    p95_ms: String,
    dropped: String,
    bars: Vec<Bar>,
    clients: Vec<ClientRow>,
    client_page: i64,
    client_pages: i64,
    attention: Vec<Attention>,
    failing: Vec<Failing>,
    iso_failures: Vec<IsoCount>,
    recent: Vec<RecentError>,
}

struct Bar {
    label: String,
    total: i64,
    failed: i64,
    width: i64,
}

struct ClientRow {
    code: String,
    name: String,
    active: bool,
    requests: i64,
    failed: i64,
}

struct Attention {
    code: String,
    name: String,
    path: String,
    error: String,
    when: String,
}

struct Failing {
    path: String,
    count: i64,
}

struct IsoCount {
    code: String,
    count: i64,
}

struct RecentError {
    href: String,
    when: String,
    path: String,
    client: String,
    status: i32,
    iso: String,
    error: String,
}

const CLIENT_PAGE: i64 = 10;

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<DashQuery>,
) -> Response {
    let start = Utc::now().date_naive().and_hms_opt(0, 0, 0).expect("midnight").and_utc();
    let end = start + Duration::days(1);
    match load(&state, &user, &query, start, end).await {
        Ok(page) => html(page),
        Err(err) => db_err(err),
    }
}

async fn load(
    state: &AppState,
    user: &CurrentUser,
    query: &DashQuery,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<DashboardPage, sqlx::Error> {
    let pool = &state.db;
    let requests: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM request_logs WHERE requested_at >= $1 AND requested_at < $2",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await?;
    let failed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM request_logs WHERE requested_at >= $1 AND requested_at < $2 AND is_error",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await?;
    let avg_ms: f64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(duration_ms), 0)::float8 FROM request_logs WHERE requested_at >= $1 AND requested_at < $2",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await?;
    let p95_ms: f64 = sqlx::query_scalar(
        "SELECT COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms), 0)::float8
         FROM request_logs WHERE requested_at >= $1 AND requested_at < $2",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await?;

    let stat_rows = sqlx::query(
        "SELECT client_code, COUNT(*)::bigint AS total, COUNT(*) FILTER (WHERE is_error)::bigint AS failed
         FROM request_logs
         WHERE requested_at >= $1 AND requested_at < $2
           AND client_code IS NOT NULL AND client_code <> ''
         GROUP BY client_code
         ORDER BY total DESC",
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    let mut stats = HashMap::new();
    let mut bars = Vec::new();
    let max = stat_rows.first().map(|row| row.get::<i64, _>("total")).unwrap_or(0).max(1);
    for row in &stat_rows {
        let code: String = row.get("client_code");
        let total: i64 = row.get("total");
        let failed_count: i64 = row.get("failed");
        stats.insert(code.clone(), (total, failed_count));
        bars.push(Bar { label: code, total, failed: failed_count, width: total * 100 / max });
    }

    let clients = sqlx::query_as::<_, ClientRecord>("SELECT id, code, name, is_active FROM clients ORDER BY code")
        .fetch_all(pool)
        .await?;
    let mut client_rows: Vec<ClientRow> = clients
        .iter()
        .map(|client| {
            let (requests, failed) = stats.get(&client.code).copied().unwrap_or((0, 0));
            ClientRow {
                code: client.code.clone(),
                name: client.name.clone(),
                active: client.is_active,
                requests,
                failed,
            }
        })
        .collect();
    client_rows.sort_by(|a, b| b.requests.cmp(&a.requests).then_with(|| a.code.cmp(&b.code)));
    let client_len = client_rows.len() as i64;
    let client_pages = if client_len == 0 { 1 } else { (client_len + CLIENT_PAGE - 1) / CLIENT_PAGE };
    let mut client_page = query.client_page.unwrap_or(1).max(1).min(client_pages);
    if client_rows.is_empty() {
        client_page = 1;
    }
    let start_idx = ((client_page - 1) * CLIENT_PAGE) as usize;
    let clients_page = client_rows.into_iter().skip(start_idx).take(CLIENT_PAGE as usize).collect();

    let failing_rows = sqlx::query(
        "SELECT path, COUNT(*)::bigint AS total FROM request_logs
         WHERE requested_at >= $1 AND requested_at < $2 AND is_error
         GROUP BY path ORDER BY total DESC LIMIT 5",
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    let failing = failing_rows
        .into_iter()
        .map(|row| Failing { path: row.get("path"), count: row.get("total") })
        .collect();

    let iso_rows = sqlx::query(
        "SELECT COALESCE(iso_response_code, '—') AS code, COUNT(*)::bigint AS total
         FROM request_logs
         WHERE requested_at >= $1 AND requested_at < $2 AND is_error
         GROUP BY iso_response_code ORDER BY total DESC",
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    let iso_failures = iso_rows
        .into_iter()
        .map(|row| IsoCount { code: row.get("code"), count: row.get("total") })
        .collect();

    let recent_rows = sqlx::query(
        "SELECT id, requested_at, path, client_code, status_code, error_message, iso_response_code
         FROM request_logs WHERE is_error ORDER BY requested_at DESC LIMIT 10",
    )
    .fetch_all(pool)
    .await?;
    let recent = recent_rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let when: DateTime<Utc> = row.get("requested_at");
            RecentError {
                href: format!("/admin/logs/{id}"),
                when: when.format("%Y-%m-%d %H:%M:%S").to_string(),
                path: row.get("path"),
                client: row.get::<Option<String>, _>("client_code").unwrap_or_else(|| "—".to_string()),
                status: row.get("status_code"),
                iso: row.get::<Option<String>, _>("iso_response_code").unwrap_or_else(|| "—".to_string()),
                error: row.get::<Option<String>, _>("error_message").unwrap_or_else(|| "Error".to_string()),
            }
        })
        .collect();

    let attention = needs_attention(pool, &clients, start - Duration::days(7)).await?;
    let success = if requests == 0 {
        "100".to_string()
    } else {
        format!("{:.1}", (requests - failed) as f64 * 100.0 / requests as f64)
    };

    Ok(DashboardPage {
        shell: Shell::new(user, "dashboard", query.notice.clone()),
        today: start.format("%Y-%m-%d").to_string(),
        requests: requests.to_string(),
        failed: failed.to_string(),
        success,
        avg_ms: format!("{avg_ms:.0}"),
        p95_ms: format!("{p95_ms:.0}"),
        dropped: state.logs.dropped().to_string(),
        bars,
        clients: clients_page,
        client_page,
        client_pages,
        attention,
        failing,
        iso_failures,
        recent,
    })
}

async fn needs_attention(
    pool: &sqlx::PgPool,
    clients: &[ClientRecord],
    since: DateTime<Utc>,
) -> Result<Vec<Attention>, sqlx::Error> {
    let active: Vec<&ClientRecord> = clients.iter().filter(|client| client.is_active).collect();
    if active.is_empty() {
        return Ok(Vec::new());
    }
    let codes: Vec<String> = active.iter().map(|client| client.code.clone()).collect();
    let recent: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT client_code FROM request_logs
         WHERE requested_at >= $1 AND client_code = ANY($2)",
    )
    .bind(since)
    .bind(&codes)
    .fetch_all(pool)
    .await?;
    let names: HashMap<&str, &str> = active.iter().map(|client| (client.code.as_str(), client.name.as_str())).collect();
    let mut items = Vec::new();
    for code in recent {
        let last = sqlx::query(
            "SELECT is_error, path, error_message, requested_at FROM request_logs
             WHERE client_code = $1 ORDER BY requested_at DESC LIMIT 5",
        )
        .bind(&code)
        .fetch_all(pool)
        .await?;
        if last.len() < 5 || last.iter().any(|row| !row.get::<bool, _>("is_error")) {
            continue;
        }
        let latest = &last[0];
        let when: DateTime<Utc> = latest.get("requested_at");
        items.push(Attention {
            name: names.get(code.as_str()).copied().unwrap_or("").to_string(),
            code,
            path: latest.get("path"),
            error: latest.get::<Option<String>, _>("error_message").unwrap_or_else(|| "Error".to_string()),
            when: when.format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }
    items.sort_by(|a, b| b.when.cmp(&a.when));
    Ok(items)
}

#[derive(sqlx::FromRow)]
struct ClientRecord {
    #[allow(dead_code)]
    id: Uuid,
    code: String,
    name: String,
    is_active: bool,
}
