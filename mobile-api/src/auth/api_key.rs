use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::logging;
use crate::state::AppState;

#[derive(Clone, Debug)]
pub struct ClientIdentity {
    pub id: Uuid,
    pub code: String,
}

#[derive(Clone)]
pub struct KeyStore {
    inner: Arc<ArcSwap<HashMap<String, ClientIdentity>>>,
}

impl KeyStore {
    pub fn from_pairs(pairs: Vec<(&str, &str)>) -> Self {
        let mut map = HashMap::new();
        for (key, code) in pairs {
            map.insert(
                sha256_hex(key),
                ClientIdentity { id: Uuid::new_v4(), code: code.to_string() },
            );
        }
        Self { inner: Arc::new(ArcSwap::from_pointee(map)) }
    }

    pub async fn load(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let store = Self { inner: Arc::new(ArcSwap::from_pointee(HashMap::new())) };
        store.refresh(pool).await?;
        Ok(store)
    }

    pub async fn refresh(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let rows = sqlx::query_as::<_, KeyRow>(
            "SELECT api_key_hash, id, code FROM clients WHERE is_active = true",
        )
        .fetch_all(pool)
        .await?;
        let map = rows
            .into_iter()
            .map(|row| (row.api_key_hash, ClientIdentity { id: row.id, code: row.code }))
            .collect();
        self.inner.store(Arc::new(map));
        Ok(())
    }

    pub fn resolve(&self, raw_key: &str) -> Option<ClientIdentity> {
        if raw_key.is_empty() {
            return None;
        }
        self.inner.load().get(&sha256_hex(raw_key)).cloned()
    }
}

#[derive(sqlx::FromRow)]
struct KeyRow {
    api_key_hash: String,
    id: Uuid,
    code: String,
}

pub fn sha256_hex(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex::encode(digest)
}

pub fn generate_api_key() -> String {
    use base64::Engine;
    let mut bytes = [0u8; 24];
    getrandom::getrandom(&mut bytes).expect("os random");
    base64::engine::general_purpose::STANDARD
        .encode(bytes)
        .trim_end_matches('=')
        .replace('+', "x")
        .replace('/', "y")
}

pub async fn require_api_key(State(state): State<AppState>, mut req: Request<Body>, next: Next) -> Response {
    let key = header_str(req.headers(), "authorization");
    let tenant = header_str(req.headers(), "tenant");
    match state.keys.resolve(&key) {
        None => StatusCode::UNAUTHORIZED.into_response(),
        Some(client) if client.code != tenant => StatusCode::BAD_REQUEST.into_response(),
        Some(client) => {
            logging::set_client(client.id, &client.code);
            req.extensions_mut().insert(client);
            next.run(req).await
        }
    }
}

fn header_str(headers: &axum::http::HeaderMap, name: &str) -> String {
    headers.get(name).and_then(|value| value.to_str().ok()).unwrap_or("").trim().to_string()
}
