use askama::Template;
use axum::extract::{Extension, Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{generate_api_key, sha256_hex, CurrentUser};
use crate::state::AppState;

use super::{db_err, forbid, html, Shell};

#[derive(Deserialize, Default)]
pub struct NoticeQuery {
    notice: Option<String>,
}

#[derive(Deserialize)]
pub struct ClientForm {
    code: String,
    name: String,
    is_active: Option<String>,
    regenerate: Option<String>,
}

#[derive(Template)]
#[template(path = "clients.html")]
struct ClientsPage {
    shell: Shell,
    clients: Vec<ClientLine>,
}

struct ClientLine {
    id: Uuid,
    code: String,
    name: String,
    api_key: String,
    active: bool,
    updated: String,
}

#[derive(Template)]
#[template(path = "client_form.html")]
struct ClientFormPage {
    shell: Shell,
    title: String,
    action: String,
    code: String,
    name: String,
    active: bool,
    show_regenerate: bool,
    error: String,
}

#[derive(Template)]
#[template(path = "client_key.html")]
struct ClientKeyPage {
    shell: Shell,
    code: String,
    api_key: String,
    created: bool,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<NoticeQuery>,
) -> Response {
    let rows = match sqlx::query_as::<_, ClientRow>(
        "SELECT id, code, name, is_active, updated_at,
                CASE WHEN $1 THEN api_key ELSE NULL END AS api_key
         FROM clients ORDER BY code",
    )
    .bind(user.is_admin())
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_err(err),
    };
    html(ClientsPage {
        shell: Shell::new(&user, "clients", query.notice),
        clients: rows
            .into_iter()
            .map(|row| ClientLine {
                id: row.id,
                code: row.code,
                name: row.name,
                api_key: row.api_key.unwrap_or_default(),
                active: row.is_active,
                updated: row.updated_at.format("%Y-%m-%d %H:%M").to_string(),
            })
            .collect(),
    })
}

pub async fn create_form(Extension(user): Extension<CurrentUser>) -> Response {
    if !user.is_admin() {
        return forbid();
    }
    html(empty_form(&user, "New client", "/admin/clients/new", false, String::new()))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Form(form): Form<ClientForm>,
) -> Response {
    if !user.is_admin() {
        return forbid();
    }
    let code = form.code.trim();
    let name = form.name.trim();
    if let Err(error) = validate(code, name) {
        return html(form_page(&user, "New client", "/admin/clients/new", code, name, form.is_active.is_some(), false, error));
    }
    match client_exists(&state, code, None).await {
        Ok(true) => {
            return html(form_page(&user, "New client", "/admin/clients/new", code, name, form.is_active.is_some(), false, "Client code already exists."));
        }
        Ok(false) => {}
        Err(err) => return db_err(err),
    }
    let api_key = generate_api_key();
    let now = Utc::now();
    let result = sqlx::query(
        "INSERT INTO clients (id, code, name, api_key_hash, api_key, is_active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
    )
    .bind(Uuid::new_v4())
    .bind(code)
    .bind(name)
    .bind(sha256_hex(&api_key))
    .bind(&api_key)
    .bind(form.is_active.is_some())
    .bind(now)
    .execute(&state.db)
    .await;
    if let Err(err) = result {
        return db_err(err);
    }
    if let Err(err) = state.keys.refresh(&state.db).await {
        tracing::error!(?err, "client key cache refresh failed");
    }
    html(ClientKeyPage {
        shell: Shell::new(&user, "clients", None),
        code: code.to_string(),
        api_key,
        created: true,
    })
}

pub async fn edit_form(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Response {
    if !user.is_admin() {
        return forbid();
    }
    match load_client(&state, id).await {
        Ok(Some(client)) => html(form_page(
            &user,
            "Edit client",
            &format!("/admin/clients/{id}/edit"),
            &client.code,
            &client.name,
            client.is_active,
            true,
            String::new(),
        )),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => db_err(err),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Form(form): Form<ClientForm>,
) -> Response {
    if !user.is_admin() {
        return forbid();
    }
    let existing = match load_client(&state, id).await {
        Ok(Some(existing)) => existing,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(err) => return db_err(err),
    };
    let code = form.code.trim();
    let name = form.name.trim();
    let action = format!("/admin/clients/{id}/edit");
    if let Err(error) = validate(code, name) {
        return html(form_page(&user, "Edit client", &action, code, name, form.is_active.is_some(), true, error));
    }
    match client_exists(&state, code, Some(id)).await {
        Ok(true) => {
            return html(form_page(&user, "Edit client", &action, code, name, form.is_active.is_some(), true, "Client code already exists."));
        }
        Ok(false) => {}
        Err(err) => return db_err(err),
    }
    let regenerate = form.regenerate.is_some();
    let api_key = regenerate.then(generate_api_key);
    let hash = api_key.as_deref().map(sha256_hex).unwrap_or_else(|| existing.api_key_hash.clone());
    let stored_key = api_key.clone().or(existing.api_key.clone());
    let result = sqlx::query(
        "UPDATE clients SET code = $2, name = $3, api_key_hash = $4, api_key = $5, is_active = $6, updated_at = $7 WHERE id = $1",
    )
    .bind(id)
    .bind(code)
    .bind(name)
    .bind(hash)
    .bind(stored_key)
    .bind(form.is_active.is_some())
    .bind(Utc::now())
    .execute(&state.db)
    .await;
    if let Err(err) = result {
        return db_err(err);
    }
    if let Err(err) = state.keys.refresh(&state.db).await {
        tracing::error!(?err, "client key cache refresh failed");
    }
    if let Some(api_key) = api_key {
        return html(ClientKeyPage {
            shell: Shell::new(&user, "clients", None),
            code: code.to_string(),
            api_key,
            created: false,
        });
    }
    Redirect::to("/admin/clients?notice=Client+updated").into_response()
}

fn validate(code: &str, name: &str) -> Result<(), String> {
    if code.is_empty() || code.len() > 100 {
        return Err("Client code is required (max 100).".to_string());
    }
    if !code.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
        return Err("Client code may contain letters, numbers, hyphen, and underscore.".to_string());
    }
    if name.is_empty() || name.len() > 200 {
        return Err("Name is required (max 200).".to_string());
    }
    Ok(())
}

async fn client_exists(state: &AppState, code: &str, except: Option<Uuid>) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE code = $1 AND ($2::uuid IS NULL OR id <> $2)")
        .bind(code)
        .bind(except)
        .fetch_one(&state.db)
        .await?;
    Ok(count > 0)
}

async fn load_client(state: &AppState, id: Uuid) -> Result<Option<ClientRowFull>, sqlx::Error> {
    sqlx::query_as("SELECT id, code, name, api_key_hash, api_key, is_active FROM clients WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
}

fn empty_form(user: &CurrentUser, title: &str, action: &str, show_regenerate: bool, error: String) -> ClientFormPage {
    form_page(user, title, action, "", "", true, show_regenerate, error)
}

fn form_page(
    user: &CurrentUser,
    title: &str,
    action: &str,
    code: &str,
    name: &str,
    active: bool,
    show_regenerate: bool,
    error: impl Into<String>,
) -> ClientFormPage {
    ClientFormPage {
        shell: Shell::new(user, "clients", None),
        title: title.to_string(),
        action: action.to_string(),
        code: code.to_string(),
        name: name.to_string(),
        active,
        show_regenerate,
        error: error.into(),
    }
}

#[derive(sqlx::FromRow)]
struct ClientRow {
    id: Uuid,
    code: String,
    name: String,
    is_active: bool,
    updated_at: chrono::DateTime<Utc>,
    api_key: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ClientRowFull {
    #[allow(dead_code)]
    id: Uuid,
    code: String,
    name: String,
    api_key_hash: String,
    api_key: Option<String>,
    is_active: bool,
}
