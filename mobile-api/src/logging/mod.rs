mod extract;
mod middleware;
mod query;
mod repo;
mod writer;

use std::sync::{Arc, Mutex};

use uuid::Uuid;

pub use extract::{extract_fields, Extracted};
pub use middleware::log_api;
pub use query::{build_clauses, clauses_sql, LogFilter};
pub use repo::{log_by_id, related_logs, search_logs, LogRow, LogSummary};
pub use writer::{LogBus, LogRecord, LogWorker};

tokio::task_local! {
    static LOG_SLOT: Arc<Mutex<LogSlot>>;
}

#[derive(Clone, Default)]
struct LogSlot {
    client_id: Option<Uuid>,
    client_code: Option<String>,
    upstream_path: Option<String>,
    upstream_status: Option<i32>,
    upstream_duration_ms: Option<i32>,
    upstream_body: Option<String>,
    error_message: Option<String>,
}

fn new_slot() -> Arc<Mutex<LogSlot>> {
    Arc::new(Mutex::new(LogSlot::default()))
}

pub fn set_client(id: Uuid, code: &str) {
    with_slot(|slot| {
        slot.client_id = Some(id);
        slot.client_code = Some(code.to_string());
    });
}

pub fn set_upstream(path: &str, status: i32, duration_ms: i32, body: &str) {
    with_slot(|slot| {
        slot.upstream_path = Some(path.to_string());
        slot.upstream_status = Some(status);
        slot.upstream_duration_ms = Some(duration_ms);
        slot.upstream_body = Some(body.to_string());
    });
}

pub fn set_error(message: &str) {
    with_slot(|slot| {
        slot.error_message = Some(message.to_string());
    });
}

fn with_slot(update: impl FnOnce(&mut LogSlot)) {
    let _ = LOG_SLOT.try_with(|slot| {
        if let Ok(mut guard) = slot.lock() {
            update(&mut guard);
        }
    });
}

fn snapshot(slot: &Arc<Mutex<LogSlot>>) -> LogSlot {
    slot.lock().unwrap_or_else(|err| err.into_inner()).clone()
}
