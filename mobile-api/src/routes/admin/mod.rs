mod auth;
mod clients;
mod dashboard;
mod logs;
mod users;

use askama::Template;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;

use crate::auth::{require_admin, CurrentUser};
use crate::state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/login", get(auth::login_form).post(auth::login_submit))
        .route("/logout", post(auth::logout));

    let private = Router::new()
        .route("/", get(dashboard::index))
        .route("/dashboard", get(dashboard::index))
        .route("/logs", get(logs::index))
        .route("/logs/table", get(logs::table))
        .route("/logs/{id}", get(logs::detail))
        .route("/clients", get(clients::index))
        .route("/clients/new", get(clients::create_form).post(clients::create))
        .route("/clients/{id}", get(clients::edit_form).post(clients::update))
        .route("/clients/{id}/edit", get(clients::edit_form).post(clients::update))
        .route("/clients/{id}/rotate-key", post(clients::rotate_key))
        .route("/users", get(users::index))
        .route("/users/new", get(users::create_form).post(users::create))
        .route("/users/{id}/edit", get(users::edit_form).post(users::update))
        .route_layer(middleware::from_fn_with_state(state, require_admin));

    public.merge(private)
}

pub struct Shell {
    pub display_name: String,
    pub role_label: String,
    pub admin: bool,
    pub notice: String,
    pub nav: String,
}

impl Shell {
    pub fn new(user: &CurrentUser, nav: &str, notice: Option<String>) -> Self {
        Self {
            display_name: user.display_name.clone(),
            role_label: user.role.label().to_string(),
            admin: user.is_admin(),
            notice: notice.unwrap_or_default(),
            nav: nav.to_string(),
        }
    }
}

pub fn forbid() -> Response {
    (StatusCode::FORBIDDEN, "Only an admin can do that.").into_response()
}

pub struct Choice {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

pub fn html(template: impl Template) -> Response {
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(err) => {
            tracing::error!(?err, "template render failed");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
        }
    }
}

pub fn db_err(err: sqlx::Error) -> Response {
    tracing::error!(?err, "admin query failed");
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
}

pub fn choices(options: &[(&str, &str)], current: &str) -> Vec<Choice> {
    options
        .iter()
        .map(|(value, label)| Choice {
            value: (*value).to_string(),
            label: (*label).to_string(),
            selected: *value == current,
        })
        .collect()
}
