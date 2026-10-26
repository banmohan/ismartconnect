use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct LogFilter {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub preset: Option<String>,
    pub path: Option<String>,
    pub client_code: Option<String>,
    pub http_method: Option<String>,
    pub status_code: Option<i32>,
    pub status_class: Option<i32>,
    pub iso_response_code: Option<String>,
    pub is_error: Option<bool>,
    pub min_duration_ms: Option<i32>,
    pub max_duration_ms: Option<i32>,
    pub account: Option<String>,
    pub member_id: Option<String>,
    pub tran_code: Option<String>,
    pub search: Option<String>,
    pub sort_by: String,
    pub sort_dir: String,
    pub page: i64,
    pub page_size: i64,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            preset: None,
            path: None,
            client_code: None,
            http_method: None,
            status_code: None,
            status_class: None,
            iso_response_code: None,
            is_error: None,
            min_duration_ms: None,
            max_duration_ms: None,
            account: None,
            member_id: None,
            tran_code: None,
            search: None,
            sort_by: "requested_at".to_string(),
            sort_dir: "desc".to_string(),
            page: 1,
            page_size: 25,
        }
    }
}

pub enum Clause {
    Raw(&'static str),
    Text(String),
    Int(i32),
    Bool(bool),
    Time(DateTime<Utc>),
}

impl LogFilter {
    pub fn time_range(&self, now: DateTime<Utc>) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
        match self.preset.as_deref() {
            Some("15m") => (Some(now - chrono::Duration::minutes(15)), Some(now)),
            Some("1h") => (Some(now - chrono::Duration::hours(1)), Some(now)),
            Some("today") => {
                let start = now.date_naive().and_hms_opt(0, 0, 0).expect("midnight").and_utc();
                (Some(start), Some(now))
            }
            Some("7d") => (Some(now - chrono::Duration::days(7)), Some(now)),
            _ => (self.from, self.to),
        }
    }

    pub fn order_sql(&self) -> &'static str {
        let asc = self.sort_dir.eq_ignore_ascii_case("asc");
        match self.sort_by.as_str() {
            "duration_ms" => {
                if asc { "duration_ms ASC" } else { "duration_ms DESC" }
            }
            "status_code" => {
                if asc { "status_code ASC" } else { "status_code DESC" }
            }
            "path" => {
                if asc { "path ASC" } else { "path DESC" }
            }
            "client_code" => {
                if asc { "client_code ASC" } else { "client_code DESC" }
            }
            "http_method" => {
                if asc { "http_method ASC" } else { "http_method DESC" }
            }
            "iso_response_code" => {
                if asc { "iso_response_code ASC" } else { "iso_response_code DESC" }
            }
            _ => {
                if asc { "requested_at ASC" } else { "requested_at DESC" }
            }
        }
    }
}

pub fn build_clauses(filter: &LogFilter, now: DateTime<Utc>) -> Vec<Clause> {
    let mut clauses = Vec::new();
    let (from, to) = filter.time_range(now);
    if let Some(from) = from {
        clauses.push(Clause::Raw(" AND requested_at >= "));
        clauses.push(Clause::Time(from));
    }
    if let Some(to) = to {
        clauses.push(Clause::Raw(" AND requested_at <= "));
        clauses.push(Clause::Time(to));
    }
    if let Some(path) = clean(&filter.path) {
        clauses.push(Clause::Raw(" AND path ILIKE "));
        clauses.push(Clause::Text(like_contains(&path)));
    }
    if let Some(client) = clean(&filter.client_code) {
        clauses.push(Clause::Raw(" AND client_code = "));
        clauses.push(Clause::Text(client));
    }
    if let Some(method) = clean(&filter.http_method) {
        clauses.push(Clause::Raw(" AND http_method = "));
        clauses.push(Clause::Text(method.to_ascii_uppercase()));
    }
    if let Some(status) = filter.status_code {
        clauses.push(Clause::Raw(" AND status_code = "));
        clauses.push(Clause::Int(status));
    }
    if let Some(class) = filter.status_class.filter(|class| (2..=5).contains(class)) {
        clauses.push(Clause::Raw(" AND status_code >= "));
        clauses.push(Clause::Int(class * 100));
        clauses.push(Clause::Raw(" AND status_code < "));
        clauses.push(Clause::Int((class + 1) * 100));
    }
    if let Some(iso) = clean(&filter.iso_response_code) {
        clauses.push(Clause::Raw(" AND iso_response_code = "));
        clauses.push(Clause::Text(iso));
    }
    if let Some(is_error) = filter.is_error {
        clauses.push(Clause::Raw(" AND is_error = "));
        clauses.push(Clause::Bool(is_error));
    }
    if let Some(min) = filter.min_duration_ms {
        clauses.push(Clause::Raw(" AND duration_ms >= "));
        clauses.push(Clause::Int(min));
    }
    if let Some(max) = filter.max_duration_ms {
        clauses.push(Clause::Raw(" AND duration_ms <= "));
        clauses.push(Clause::Int(max));
    }
    if let Some(account) = clean(&filter.account) {
        clauses.push(Clause::Raw(" AND (account_number = "));
        clauses.push(Clause::Text(account.clone()));
        clauses.push(Clause::Raw(" OR src_account = "));
        clauses.push(Clause::Text(account.clone()));
        clauses.push(Clause::Raw(" OR dest_account = "));
        clauses.push(Clause::Text(account));
        clauses.push(Clause::Raw(")"));
    }
    if let Some(member) = clean(&filter.member_id) {
        clauses.push(Clause::Raw(" AND member_id = "));
        clauses.push(Clause::Text(member));
    }
    if let Some(tran) = clean(&filter.tran_code) {
        clauses.push(Clause::Raw(" AND tran_code = "));
        clauses.push(Clause::Text(tran));
    }
    if let Some(search) = clean(&filter.search) {
        clauses.push(Clause::Raw(" AND search_text ILIKE "));
        clauses.push(Clause::Text(like_contains(&search)));
    }
    clauses
}

pub fn clauses_sql(clauses: &[Clause]) -> String {
    let mut sql = String::from("WHERE 1=1");
    let mut index = 1;
    for clause in clauses {
        match clause {
            Clause::Raw(text) => sql.push_str(text),
            Clause::Text(_) | Clause::Int(_) | Clause::Bool(_) | Clause::Time(_) => {
                sql.push('$');
                sql.push_str(&index.to_string());
                index += 1;
            }
        }
    }
    sql
}

fn clean(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn like_contains(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    format!("%{escaped}%")
}

#[cfg(test)]
mod tests {
    use super::{build_clauses, clauses_sql, LogFilter};
    use chrono::Utc;

    #[test]
    fn account_and_iso_filters_use_extracted_columns() {
        let filter = LogFilter {
            account: Some("ACC1".to_string()),
            iso_response_code: Some("51".to_string()),
            search: Some("timeout".to_string()),
            ..LogFilter::default()
        };
        let sql = clauses_sql(&build_clauses(&filter, Utc::now()));
        assert!(sql.contains("iso_response_code ="));
        assert!(sql.contains("account_number ="));
        assert!(sql.contains("src_account ="));
        assert!(sql.contains("dest_account ="));
        assert!(sql.contains("search_text ILIKE"));
        assert!(!sql.to_lowercase().contains("request_body"));
        assert!(!sql.to_lowercase().contains("response_body"));
    }
}
