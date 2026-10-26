use cookie::Key;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::auth::{cookie_key, KeyStore};
use crate::cbs::CbsClient;
use crate::logging::LogBus;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cbs: CbsClient,
    pub keys: KeyStore,
    pub logs: LogBus,
    pub cookie_key: Key,
    pub cookie_secure: bool,
}

impl AppState {
    pub fn new(db: PgPool, cbs: CbsClient, keys: KeyStore, logs: LogBus, secret: &str, cookie_secure: bool) -> Self {
        Self { db, cbs, keys, logs, cookie_key: cookie_key(secret), cookie_secure }
    }

    pub fn for_tests(cbs: CbsClient, keys: KeyStore) -> Self {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@127.0.0.1:9/mobile_api")
            .expect("lazy pool");
        Self::new(db, cbs, keys, LogBus::discarding(), "test-cookie-secret", false)
    }
}
