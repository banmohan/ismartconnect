use askama::Template;
use axum::extract::{Extension, Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{hash_password, CurrentUser};
use crate::state::AppState;

use super::{db_err, forbid, html, Shell};

#[derive(Deserialize, Default)]
pub struct NoticeQuery {
    notice: Option<String>,
}

#[derive(Deserialize)]
pub struct UserForm {
    username: String,
    display_name: String,
    password: Option<String>,
    role: Option<String>,
    is_active: Option<String>,
}

#[derive(Template)]
#[template(path = "users.html")]
struct UsersPage {
    shell: Shell,
    users: Vec<UserLine>,
}

struct UserLine {
    id: Uuid,
    username: String,
    display_name: String,
    role_label: String,
    active: bool,
}

#[derive(Template)]
#[template(path = "user_form.html")]
struct UserFormPage {
    shell: Shell,
    title: String,
    action: String,
    username: String,
    display_name: String,
    active: bool,
    role: String,
    password_required: bool,
    error: String,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<NoticeQuery>,
) -> Response {
    let rows = match sqlx::query_as::<_, UserRow>("SELECT id, username, display_name, is_active, role FROM users ORDER BY username")
        .fetch_all(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(err) => return db_err(err),
    };
    html(UsersPage {
        shell: Shell::new(&user, "users", query.notice),
        users: rows
            .into_iter()
            .map(|row| UserLine {
                id: row.id,
                username: row.username,
                display_name: row.display_name,
                role_label: crate::auth::Role::parse(&row.role).label().to_string(),
                active: row.is_active,
            })
            .collect(),
    })
}

pub async fn create_form(Extension(user): Extension<CurrentUser>) -> Response {
    if !user.is_admin() {
        return forbid();
    }
    html(form_page(&user, "New user", "/admin/users/new", "", "", "user", true, true, String::new()))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Form(form): Form<UserForm>,
) -> Response {
    if !user.is_admin() {
        return forbid();
    }
    let username = form.username.trim();
    let display_name = form.display_name.trim();
    let password = form.password.unwrap_or_default();
    let role = normalize_role(form.role.as_deref());
    if let Err(error) = validate(username, display_name, Some(password.trim()), true) {
        return html(form_page(&user, "New user", "/admin/users/new", username, display_name, role, form.is_active.is_some(), true, error));
    }
    match user_exists(&state, username, None).await {
        Ok(true) => {
            return html(form_page(&user, "New user", "/admin/users/new", username, display_name, role, form.is_active.is_some(), true, "Username already exists."));
        }
        Ok(false) => {}
        Err(err) => return db_err(err),
    }
    let Ok(hash) = hash_password(password.trim()) else {
        return html(form_page(&user, "New user", "/admin/users/new", username, display_name, role, form.is_active.is_some(), true, "Could not hash the password."));
    };
    let now = Utc::now();
    if let Err(err) = sqlx::query(
        "INSERT INTO users (id, username, password_hash, display_name, is_active, role, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
    )
    .bind(Uuid::new_v4())
    .bind(username)
    .bind(hash)
    .bind(display_name)
    .bind(form.is_active.is_some())
    .bind(role)
    .bind(now)
    .execute(&state.db)
    .await
    {
        return db_err(err);
    }
    Redirect::to("/admin/users?notice=User+created").into_response()
}

pub async fn edit_form(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Response {
    if !user.is_admin() {
        return forbid();
    }
    match load_user(&state, id).await {
        Ok(Some(existing)) => html(form_page(
            &user,
            "Edit user",
            &format!("/admin/users/{id}/edit"),
            &existing.username,
            &existing.display_name,
            &existing.role,
            existing.is_active,
            false,
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
    Form(form): Form<UserForm>,
) -> Response {
    if !user.is_admin() {
        return forbid();
    }
    let existing = match load_user(&state, id).await {
        Ok(Some(existing)) => existing,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(err) => return db_err(err),
    };
    let username = form.username.trim();
    let display_name = form.display_name.trim();
    let password = form.password.unwrap_or_default();
    let role = normalize_role(form.role.as_deref());
    let active = form.is_active.is_some();
    let action = format!("/admin/users/{id}/edit");
    let password_value = if password.trim().is_empty() { None } else { Some(password.trim()) };
    if let Err(error) = validate(username, display_name, password_value, false) {
        return html(form_page(&user, "Edit user", &action, username, display_name, role, active, false, error));
    }
    if existing.role == "admin" && (role != "admin" || !active) {
        match other_active_admins(&state, id).await {
            Ok(0) => {
                return html(form_page(
                    &user,
                    "Edit user",
                    &action,
                    username,
                    display_name,
                    role,
                    active,
                    false,
                    "Keep at least one active admin.",
                ));
            }
            Ok(_) => {}
            Err(err) => return db_err(err),
        }
    }
    match user_exists(&state, username, Some(id)).await {
        Ok(true) => {
            return html(form_page(&user, "Edit user", &action, username, display_name, role, active, false, "Username already exists."));
        }
        Ok(false) => {}
        Err(err) => return db_err(err),
    }
    let hash = if let Some(password) = password_value {
        match hash_password(password) {
            Ok(hash) => Some(hash),
            Err(_) => {
                return html(form_page(&user, "Edit user", &action, username, display_name, role, active, false, "Could not hash the password."));
            }
        }
    } else {
        None
    };
    let result = sqlx::query(
        "UPDATE users SET username = $2, display_name = $3, is_active = $4, role = $5, updated_at = $6,
            password_hash = COALESCE($7, password_hash)
         WHERE id = $1",
    )
    .bind(id)
    .bind(username)
    .bind(display_name)
    .bind(active)
    .bind(role)
    .bind(Utc::now())
    .bind(hash)
    .execute(&state.db)
    .await;
    if let Err(err) = result {
        return db_err(err);
    }
    Redirect::to("/admin/users?notice=User+updated").into_response()
}

fn validate(username: &str, display_name: &str, password: Option<&str>, required: bool) -> Result<(), String> {
    if username.is_empty() || username.len() > 100 {
        return Err("Username is required (max 100).".to_string());
    }
    if display_name.is_empty() || display_name.len() > 200 {
        return Err("Display name is required (max 200).".to_string());
    }
    match password {
        None if required => Err("Password is required.".to_string()),
        Some(password) if password.len() < 8 => Err("Password must be at least 8 characters.".to_string()),
        _ => Ok(()),
    }
}

async fn user_exists(state: &AppState, username: &str, except: Option<Uuid>) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = $1 AND ($2::uuid IS NULL OR id <> $2)")
        .bind(username)
        .bind(except)
        .fetch_one(&state.db)
        .await?;
    Ok(count > 0)
}

async fn load_user(state: &AppState, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, username, display_name, is_active, role FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
}

async fn other_active_admins(state: &AppState, except: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'admin' AND is_active AND id <> $1")
        .bind(except)
        .fetch_one(&state.db)
        .await
}

fn normalize_role(value: Option<&str>) -> &'static str {
    if value == Some("admin") { "admin" } else { "user" }
}

fn form_page(
    user: &CurrentUser,
    title: &str,
    action: &str,
    username: &str,
    display_name: &str,
    role: &str,
    active: bool,
    password_required: bool,
    error: impl Into<String>,
) -> UserFormPage {
    UserFormPage {
        shell: Shell::new(user, "users", None),
        title: title.to_string(),
        action: action.to_string(),
        username: username.to_string(),
        display_name: display_name.to_string(),
        role: role.to_string(),
        active,
        password_required,
        error: error.into(),
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    display_name: String,
    is_active: bool,
    role: String,
}
