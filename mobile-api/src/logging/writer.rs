use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct LogRecord {
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
    pub search_text: String,
}

#[derive(Clone)]
pub struct LogBus {
    tx: mpsc::Sender<LogRecord>,
    dropped: Arc<AtomicU64>,
}

impl LogBus {
    pub fn discarding() -> Self {
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move { while rx.recv().await.is_some() {} });
        Self { tx, dropped: Arc::new(AtomicU64::new(0)) }
    }

    pub fn try_send(&self, record: LogRecord) {
        if self.tx.try_send(record).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

pub struct LogWorker {
    tx: mpsc::Sender<LogRecord>,
    handle: JoinHandle<()>,
    dropped: Arc<AtomicU64>,
}

impl LogWorker {
    pub fn spawn(pool: PgPool) -> Self {
        let (tx, mut rx) = mpsc::channel(1024);
        let dropped = Arc::new(AtomicU64::new(0));
        let handle = tokio::spawn(async move {
            let mut batch = Vec::with_capacity(50);
            loop {
                tokio::select! {
                    biased;
                    item = rx.recv() => {
                        match item {
                            Some(row) => {
                                batch.push(row);
                                if batch.len() >= 50 {
                                    flush(&pool, &mut batch).await;
                                }
                            }
                            None => {
                                flush(&pool, &mut batch).await;
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(200)), if !batch.is_empty() => {
                        flush(&pool, &mut batch).await;
                    }
                }
            }
        });
        Self { tx, handle, dropped }
    }

    pub fn bus(&self) -> LogBus {
        LogBus { tx: self.tx.clone(), dropped: self.dropped.clone() }
    }

    pub async fn shutdown(self) {
        drop(self.tx);
        let _ = self.handle.await;
    }
}

async fn flush(pool: &PgPool, batch: &mut Vec<LogRecord>) {
    if batch.is_empty() {
        return;
    }
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!(?err, "log transaction failed");
            batch.clear();
            return;
        }
    };
    for row in batch.drain(..) {
        if let Err(err) = insert_log(&mut tx, &row).await {
            tracing::error!(?err, "log insert failed");
        }
    }
    if let Err(err) = tx.commit().await {
        tracing::error!(?err, "log commit failed");
    }
}

async fn insert_log(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, row: &LogRecord) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO request_logs (
            id, request_id, requested_at, duration_ms, http_method, path, query_string, client_id, client_code,
            ip_address, request_headers, request_body, status_code, response_body, is_error, error_message,
            exception_details, iso_response_code, account_number, src_account, dest_account, member_id, tran_code,
            amount, upstream_path, upstream_status, upstream_duration_ms, upstream_body, search_text
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9,
            $10, $11, $12, $13, $14, $15, $16,
            $17, $18, $19, $20, $21, $22, $23,
            $24, $25, $26, $27, $28, $29
        )",
    )
    .bind(row.id)
    .bind(row.request_id)
    .bind(row.requested_at)
    .bind(row.duration_ms)
    .bind(&row.http_method)
    .bind(&row.path)
    .bind(&row.query_string)
    .bind(row.client_id)
    .bind(&row.client_code)
    .bind(&row.ip_address)
    .bind(&row.request_headers)
    .bind(&row.request_body)
    .bind(row.status_code)
    .bind(&row.response_body)
    .bind(row.is_error)
    .bind(&row.error_message)
    .bind(&row.exception_details)
    .bind(&row.iso_response_code)
    .bind(&row.account_number)
    .bind(&row.src_account)
    .bind(&row.dest_account)
    .bind(&row.member_id)
    .bind(&row.tran_code)
    .bind(row.amount)
    .bind(&row.upstream_path)
    .bind(row.upstream_status)
    .bind(row.upstream_duration_ms)
    .bind(&row.upstream_body)
    .bind(&row.search_text)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
