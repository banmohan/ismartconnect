use askama::Template;
use axum::extract::{Form, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::auth::{append_cookies, jar_from_request, login_cookie, logout_cookie, read_session, verify_password};
use crate::state::AppState;

use super::html;

#[derive(Deserialize, Default)]
pub struct LoginQuery {
    next: Option<String>,
    notice: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
    next: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage {
    error: String,
    username: String,
    next: String,
    notice: String,
}

pub async fn login_form(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LoginQuery>,
) -> Response {
    if session_user(&state, &headers).await.is_some() {
        return Redirect::to("/admin").into_response();
    }
    html(LoginPage {
        error: String::new(),
        username: String::new(),
        next: safe_next(query.next.as_deref()).unwrap_or("/admin").to_string(),
        notice: query.notice.unwrap_or_default(),
    })
}

pub async fn login_submit(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    let next = safe_next(form.next.as_deref()).unwrap_or("/admin").to_string();
    let username = form.username.trim();
    let failure = LoginPage {
        error: "Invalid username or password.".to_string(),
        username: username.to_string(),
        next: next.clone(),
        notice: String::new(),
    };
    let row = sqlx::query_as::<_, UserLogin>(
        "SELECT id, password_hash, is_active FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(&state.db)
    .await;
    let Ok(Some(user)) = row else {
        return html(failure);
    };
    if !user.is_active || !verify_password(&form.password, &user.password_hash) {
        return html(failure);
    }
    let jar = login_cookie(&state.cookie_key, user.id, state.cookie_secure);
    let mut response = Redirect::to(&next).into_response();
    append_cookies(&jar, response.headers_mut());
    response
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let jar = logout_cookie(&headers, state.cookie_secure);
    let mut response = Redirect::to("/admin/login").into_response();
    append_cookies(&jar, response.headers_mut());
    response
}

async fn session_user(state: &AppState, headers: &HeaderMap) -> Option<()> {
    let jar = jar_from_request(headers);
    let (id, exp) = read_session(&jar, &state.cookie_key)?;
    if exp <= chrono::Utc::now().timestamp() {
        return None;
    }
    let active: Option<bool> = sqlx::query_scalar("SELECT is_active FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()?;
active.filter(|active| *active).map(|_| ())
}

fn safe_next(value: Option<&str>) -> Option<&str> {
    let value = value?.trim();
    if value.starts_with("/admin") && !value.starts_with("//") && !value.contains('\\') {
        Some(value)
    } else {
        None
    }
}

#[derive(sqlx::FromRow)]
struct UserLogin {
    id: uuid::Uuid,
    password_hash: String,
    is_active: bool,
}
