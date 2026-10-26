use askama::Template;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::logging::{search_logs, LogFilter, LogRow, LogSummary};
use crate::state::AppState;

use super::{choices, db_err, html, Choice, Shell};

#[derive(Deserialize, Default)]
pub struct LogQuery {
    from: Option<String>,
    to: Option<String>,
    preset: Option<String>,
    path: Option<String>,
    client_code: Option<String>,
    method: Option<String>,
    status: Option<String>,
    status_class: Option<String>,
    iso: Option<String>,
    is_error: Option<String>,
    min_ms: Option<String>,
    max_ms: Option<String>,
    account: Option<String>,
    member_id: Option<String>,
    tran_code: Option<String>,
    q: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
    notice: Option<String>,
}

#[derive(Template)]
#[template(path = "logs.html")]
struct LogsPage {
    shell: Shell,
    presets: Vec<Choice>,
    methods: Vec<Choice>,
    classes: Vec<Choice>,
    errors: Vec<Choice>,
    sorts: Vec<Choice>,
    dirs: Vec<Choice>,
    sizes: Vec<Choice>,
    clients: Vec<Choice>,
    from: String,
    to: String,
    path: String,
    status: String,
    iso: String,
    min_ms: String,
    max_ms: String,
    account: String,
    member_id: String,
    tran_code: String,
    q: String,
    table: String,
}

#[derive(Template)]
#[template(path = "logs_table.html")]
struct LogsTable {
    rows: Vec<LogLine>,
    total: i64,
    page: i64,
    pages: i64,
    links: Vec<PageLink>,
}

#[derive(Template)]
#[template(path = "log_detail.html")]
struct LogDetailPage {
    shell: Shell,
    when: String,
    duration: String,
    slow: bool,
    method: String,
    path: String,
    query: String,
    client: String,
    ip: String,
    status: i32,
    status_class: String,
    iso: String,
    request_id: String,
    error: String,
    exception: String,
    steps: Vec<Step>,
    headers: Vec<HeaderLine>,
    request_pretty: String,
    request_raw: String,
    response_pretty: String,
    response_raw: String,
    upstream_pretty: String,
    upstream_raw: String,
    curl: String,
    related: Vec<LogLine>,
}

struct LogLine {
    href: String,
    when: String,
    client: String,
    method: String,
    path: String,
    status: i32,
    status_class: String,
    iso: String,
    duration: String,
    slow: bool,
    account: String,
    error: bool,
}

struct PageLink {
    label: String,
    href: String,
    partial: String,
    current: bool,
    gap: bool,
}

struct Step {
    name: String,
    detail: String,
    status: String,
    status_class: String,
    duration: String,
}

struct HeaderLine {
    name: String,
    value: String,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<LogQuery>,
) -> Response {
    let filter = parse_filter(&query);
    let clients = match client_codes(&state).await {
        Ok(codes) => codes,
        Err(err) => return db_err(err),
    };
    let table = match render_table(&state, &filter).await {
        Ok(table) => table,
        Err(err) => return db_err(err),
    };
    html(page(&user, &query, &filter, clients, table))
}

pub async fn table(State(state): State<AppState>, Query(query): Query<LogQuery>) -> Response {
    let filter = parse_filter(&query);
    let body = match render_table(&state, &filter).await {
        Ok(table) => table,
        Err(err) => return db_err(err),
    };
    let mut response = axum::response::Html(body).into_response();
    if let Ok(value) = format!("/admin/logs?{}", query_string(&filter)).parse() {
        response.headers_mut().insert("hx-push-url", value);
    }
    response
}

pub async fn detail(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Response {
    let row = match crate::logging::log_by_id(&state.db, id).await {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(err) => return db_err(err),
    };
    let related = match crate::logging::related_logs(&state.db, &row).await {
        Ok(rows) => rows.into_iter().map(line_from_summary).collect(),
        Err(err) => return db_err(err),
    };
    html(detail_page(&user, &row, related))
}

async fn render_table(state: &AppState, filter: &LogFilter) -> Result<String, sqlx::Error> {
    let page = search_logs(&state.db, filter).await?;
    let pages = if page.total <= 0 {
        0
    } else {
        (page.total + filter.page_size - 1) / filter.page_size
    };
    let links = page_links(filter, pages);
    LogsTable {
        rows: page.rows.into_iter().map(line_from_summary).collect(),
        total: page.total,
        page: filter.page,
        pages,
        links,
    }
    .render()
    .map_err(|err| sqlx::Error::Protocol(format!("template: {err}")))
}

fn page(user: &CurrentUser, query: &LogQuery, filter: &LogFilter, clients: Vec<String>, table: String) -> LogsPage {
    let mut client_choices = vec![Choice { value: String::new(), label: "All clients".to_string(), selected: filter.client_code.is_none() }];
    for code in clients {
        let selected = filter.client_code.as_deref() == Some(code.as_str());
        client_choices.push(Choice { value: code.clone(), label: code, selected });
    }
    LogsPage {
        shell: Shell::new(user, "logs", query.notice.clone()),
        presets: choices(&[("", "Any time"), ("15m", "15 minutes"), ("1h", "1 hour"), ("today", "Today"), ("7d", "7 days")], filter.preset.as_deref().unwrap_or("")),
        methods: choices(&[("", "Any"), ("GET", "GET"), ("POST", "POST")], filter.http_method.as_deref().unwrap_or("")),
        classes: choices(&[("", "Any"), ("2", "2xx"), ("4", "4xx"), ("5", "5xx")], &filter.status_class.map(|n| n.to_string()).unwrap_or_default()),
        errors: choices(
            &[("", "All"), ("true", "Errors"), ("false", "Success")],
            match filter.is_error {
                Some(true) => "true",
                Some(false) => "false",
                None => "",
            },
        ),
        sorts: choices(
            &[
                ("requested_at", "Time"),
                ("duration_ms", "Duration"),
                ("status_code", "Status"),
                ("path", "Path"),
                ("client_code", "Client"),
                ("http_method", "Method"),
                ("iso_response_code", "ISO"),
            ],
            &filter.sort_by,
        ),
        dirs: choices(&[("desc", "Newest / high"), ("asc", "Oldest / low")], &filter.sort_dir),
        sizes: choices(&[("10", "10"), ("25", "25"), ("50", "50"), ("100", "100")], &filter.page_size.to_string()),
        clients: client_choices,
        from: filter.from.map(|value| value.format("%Y-%m-%dT%H:%M").to_string()).unwrap_or_default(),
        to: filter.to.map(|value| value.format("%Y-%m-%dT%H:%M").to_string()).unwrap_or_default(),
        path: filter.path.clone().unwrap_or_default(),
        status: filter.status_code.map(|n| n.to_string()).unwrap_or_default(),
        iso: filter.iso_response_code.clone().unwrap_or_default(),
        min_ms: filter.min_duration_ms.map(|n| n.to_string()).unwrap_or_default(),
        max_ms: filter.max_duration_ms.map(|n| n.to_string()).unwrap_or_default(),
        account: filter.account.clone().unwrap_or_default(),
        member_id: filter.member_id.clone().unwrap_or_default(),
        tran_code: filter.tran_code.clone().unwrap_or_default(),
        q: filter.search.clone().unwrap_or_default(),
        table,
    }
}

fn parse_filter(query: &LogQuery) -> LogFilter {
    let preset = query.preset.clone().filter(|value| matches!(value.as_str(), "15m" | "1h" | "today" | "7d"));
    let sort = query.sort.clone().filter(|value| {
        matches!(
            value.as_str(),
            "requested_at" | "duration_ms" | "status_code" | "path" | "client_code" | "http_method" | "iso_response_code"
        )
    });
    let page_size = match query.page_size {
        Some(10 | 25 | 50 | 100) => query.page_size.unwrap(),
        _ => 25,
    };
    LogFilter {
        from: query.from.as_deref().and_then(parse_time),
        to: query.to.as_deref().and_then(parse_time),
        preset,
        path: blank(&query.path),
        client_code: blank(&query.client_code),
        http_method: blank(&query.method).map(|value| value.to_ascii_uppercase()),
        status_code: query.status.as_deref().and_then(|value| value.parse().ok()),
        status_class: query.status_class.as_deref().and_then(|value| value.parse().ok()).filter(|class| (2..=5).contains(class)),
        iso_response_code: blank(&query.iso),
        is_error: match query.is_error.as_deref() {
            Some("true") => Some(true),
            Some("false") => Some(false),
            _ => None,
        },
        min_duration_ms: query.min_ms.as_deref().and_then(|value| value.parse().ok()),
        max_duration_ms: query.max_ms.as_deref().and_then(|value| value.parse().ok()),
        account: blank(&query.account),
        member_id: blank(&query.member_id),
        tran_code: blank(&query.tran_code),
        search: blank(&query.q),
        sort_by: sort.unwrap_or_else(|| "requested_at".to_string()),
        sort_dir: if query.dir.as_deref() == Some("asc") { "asc".to_string() } else { "desc".to_string() },
        page: query.page.unwrap_or(1).max(1),
        page_size,
    }
}

fn query_string(filter: &LogFilter) -> String {
    let mut pairs = Vec::new();
    push(&mut pairs, "from", &filter.from.map(|value| value.format("%Y-%m-%dT%H:%M").to_string()));
    push(&mut pairs, "to", &filter.to.map(|value| value.format("%Y-%m-%dT%H:%M").to_string()));
    push(&mut pairs, "preset", &filter.preset);
    push(&mut pairs, "path", &filter.path);
    push(&mut pairs, "client_code", &filter.client_code);
    push(&mut pairs, "method", &filter.http_method);
    push(&mut pairs, "status", &filter.status_code.map(|n| n.to_string()));
    push(&mut pairs, "status_class", &filter.status_class.map(|n| n.to_string()));
    push(&mut pairs, "iso", &filter.iso_response_code);
    push(&mut pairs, "is_error", &filter.is_error.map(|value| value.to_string()));
    push(&mut pairs, "min_ms", &filter.min_duration_ms.map(|n| n.to_string()));
    push(&mut pairs, "max_ms", &filter.max_duration_ms.map(|n| n.to_string()));
    push(&mut pairs, "account", &filter.account);
    push(&mut pairs, "member_id", &filter.member_id);
    push(&mut pairs, "tran_code", &filter.tran_code);
    push(&mut pairs, "q", &filter.search);
    pairs.push(("sort".to_string(), filter.sort_by.clone()));
    pairs.push(("dir".to_string(), filter.sort_dir.clone()));
    pairs.push(("page".to_string(), filter.page.to_string()));
    pairs.push(("page_size".to_string(), filter.page_size.to_string()));
    pairs
        .into_iter()
        .map(|(key, value)| format!("{}={}", encode(&key), encode(&value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn page_links(filter: &LogFilter, pages: i64) -> Vec<PageLink> {
    if pages <= 1 {
        return Vec::new();
    }
    let mut numbers = Vec::new();
    for page in 1..=pages {
        if pages > 12 && (page - filter.page).abs() > 3 && page != 1 && page != pages {
            if page == 2 || page == pages - 1 {
                numbers.push(None);
            }
            continue;
        }
        numbers.push(Some(page));
    }
    numbers
        .into_iter()
        .map(|page| match page {
            None => PageLink { label: "…".to_string(), href: String::new(), partial: String::new(), current: false, gap: true },
            Some(page) => {
                let mut link_filter = filter.clone();
                link_filter.page = page;
                let qs = query_string(&link_filter);
                PageLink {
                    label: page.to_string(),
                    href: format!("/admin/logs?{qs}"),
                    partial: format!("/admin/logs/table?{qs}"),
                    current: page == filter.page,
                    gap: false,
                }
            }
        })
        .collect()
}

fn detail_page(user: &CurrentUser, row: &LogRow, related: Vec<LogLine>) -> LogDetailPage {
    let request_raw = row.request_body.clone().unwrap_or_default();
    let response_raw = row.response_body.clone().unwrap_or_default();
    let upstream_raw = row.upstream_body.clone().unwrap_or_default();
    let upstream_status = row.upstream_status.unwrap_or(0);
    LogDetailPage {
        shell: Shell::new(user, "logs", None),
        when: row.requested_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        duration: format!("{} ms", row.duration_ms),
        slow: row.duration_ms >= 2000,
        method: row.http_method.clone(),
        path: row.path.clone(),
        query: row.query_string.clone().unwrap_or_default(),
        client: row.client_code.clone().unwrap_or_else(|| "—".to_string()),
        ip: row.ip_address.clone().unwrap_or_else(|| "—".to_string()),
        status: row.status_code,
        status_class: status_class(row.status_code).to_string(),
        iso: row.iso_response_code.clone().unwrap_or_else(|| "—".to_string()),
        request_id: row.request_id.to_string(),
        error: row.error_message.clone().unwrap_or_default(),
        exception: row.exception_details.clone().unwrap_or_default(),
        steps: vec![
            Step {
                name: "MobileApi".to_string(),
                detail: format!("{} {}", row.http_method, row.path),
                status: row.status_code.to_string(),
                status_class: status_class(row.status_code).to_string(),
                duration: format!("{} ms", row.duration_ms),
            },
            Step {
                name: "PremiumCBS".to_string(),
                detail: row.upstream_path.clone().unwrap_or_else(|| "not called".to_string()),
                status: if upstream_status == 0 { "—".to_string() } else { upstream_status.to_string() },
                status_class: if upstream_status == 0 { "muted".to_string() } else { status_class(upstream_status).to_string() },
                duration: row.upstream_duration_ms.map(|ms| format!("{ms} ms")).unwrap_or_else(|| "—".to_string()),
            },
        ],
        headers: header_lines(row.request_headers.as_deref().unwrap_or("")),
        request_pretty: pretty(&request_raw),
        request_raw,
        response_pretty: pretty(&response_raw),
        response_raw,
        upstream_pretty: pretty(&upstream_raw),
        upstream_raw,
        curl: to_curl(row),
        related,
    }
}

fn line_from_summary(row: LogSummary) -> LogLine {
    let account = account_label(&row);
    LogLine {
        href: format!("/admin/logs/{}", row.id),
        when: row.requested_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        client: row.client_code.unwrap_or_else(|| "—".to_string()),
        method: row.http_method,
        path: row.path,
        status: row.status_code,
        status_class: status_class(row.status_code).to_string(),
        iso: row.iso_response_code.unwrap_or_else(|| "—".to_string()),
        duration: format!("{}", row.duration_ms),
        slow: row.duration_ms >= 2000,
        account,
        error: row.is_error,
    }
}

fn account_label(row: &LogSummary) -> String {
    if let Some(account) = row.account_number.as_ref().filter(|value| !value.is_empty()) {
        return account.clone();
    }
    match (row.src_account.as_ref(), row.dest_account.as_ref()) {
        (Some(src), Some(dest)) if !src.is_empty() || !dest.is_empty() => format!("{src} → {dest}"),
        (Some(src), _) if !src.is_empty() => src.clone(),
        _ => row.member_id.clone().unwrap_or_default(),
    }
}

fn status_class(code: i32) -> &'static str {
    match code {
        200..=299 => "ok",
        400..=499 => "warn",
        _ => "bad",
    }
}

fn pretty(raw: &str) -> String {
    if raw.trim().is_empty() {
        return "—".to_string();
    }
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| raw.to_string()),
        Err(_) => raw.to_string(),
    }
}

fn header_lines(raw: &str) -> Vec<HeaderLine> {
    raw.lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some(HeaderLine { name: name.trim().to_string(), value: value.trim().to_string() })
        })
        .collect()
}

fn to_curl(row: &LogRow) -> String {
    let host = header_lines(row.request_headers.as_deref().unwrap_or(""))
        .into_iter()
        .find(|line| line.name.eq_ignore_ascii_case("host"))
        .map(|line| line.value)
        .unwrap_or_else(|| "localhost:8080".to_string());
    let query = row.query_string.clone().unwrap_or_default();
    let mut lines = vec![format!("curl -X {} 'http://{host}{}{query}'", row.http_method, row.path)];
    for header in header_lines(row.request_headers.as_deref().unwrap_or("")) {
        if header.name.eq_ignore_ascii_case("host") {
            continue;
        }
        lines.push(format!("  -H '{}: {}'", escape_squote(&header.name), escape_squote(&header.value)));
    }
    if let Some(body) = row.request_body.as_ref().filter(|body| !body.is_empty()) {
        lines.push(format!("  --data-raw '{}'", escape_squote(body)));
    }
    lines.join(" \\\n")
}

fn escape_squote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

fn parse_time(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Utc));
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
        .ok()
        .map(|naive| naive.and_utc())
}

fn blank(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn push(pairs: &mut Vec<(String, String)>, key: &str, value: &Option<String>) {
    if let Some(value) = value.as_ref().filter(|value| !value.is_empty()) {
        pairs.push((key.to_string(), value.clone()));
    }
}

fn encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

async fn client_codes(state: &AppState) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT code FROM clients ORDER BY code").fetch_all(&state.db).await
}
