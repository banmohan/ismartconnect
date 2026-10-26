use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::Utc;
use cookie::{Cookie, CookieJar, Key, SameSite};
use sha2::{Digest, Sha512};
use uuid::Uuid;

use crate::state::AppState;

pub const SESSION_COOKIE: &str = "mobile_api_session";
const SESSION_SECONDS: i64 = 8 * 60 * 60;
const SLIDE_IF_REMAINING_UNDER: i64 = 4 * 60 * 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn parse(value: &str) -> Self {
        if value.eq_ignore_ascii_case("admin") { Self::Admin } else { Self::User }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Admin => "Admin",
            Self::User => "User",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
}

impl CurrentUser {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }
}

pub fn cookie_key(secret: &str) -> Key {
    Key::from(Sha512::digest(secret.as_bytes()).as_slice())
}

pub async fn require_admin(State(state): State<AppState>, req: Request<Body>, next: Next) -> Response {
    let mut jar = jar_from_request(req.headers());
    let Some((user_id, exp)) = read_session(&jar, &state.cookie_key) else {
        return Redirect::to("/admin/login").into_response();
    };
    let now = Utc::now().timestamp();
    if exp <= now {
        return Redirect::to("/admin/login").into_response();
    }
    let user = match load_user(&state, user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return Redirect::to("/admin/login").into_response(),
        Err(err) => {
            tracing::error!(?err, "admin session lookup failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let mut req = req;
    req.extensions_mut().insert(user);
    let mut response = next.run(req).await;
    if exp - now < SLIDE_IF_REMAINING_UNDER {
        write_session(&mut jar, &state.cookie_key, user_id, now + SESSION_SECONDS, state.cookie_secure);
        append_cookies(&jar, response.headers_mut());
    }
    response
}

pub fn login_cookie(key: &Key, user_id: Uuid, secure: bool) -> CookieJar {
    let mut jar = CookieJar::new();
    write_session(&mut jar, key, user_id, Utc::now().timestamp() + SESSION_SECONDS, secure);
    jar
}

pub fn logout_cookie(headers: &axum::http::HeaderMap, secure: bool) -> CookieJar {
    let mut jar = jar_from_request(headers);
    // `CookieJar::remove` emits a clearing Set-Cookie only when an original
    // cookie of the same name is already in the jar.
    if jar.get(SESSION_COOKIE).is_none() {
        jar.add_original((SESSION_COOKIE, ""));
    }
    let mut builder = Cookie::build((SESSION_COOKIE, ""))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/admin");
    if secure {
        builder = builder.secure(true);
    }
    jar.remove(builder.build());
    jar
}

pub fn append_cookies(jar: &CookieJar, headers: &mut axum::http::HeaderMap) {
    for cookie in jar.delta() {
        if let Ok(value) = cookie.to_string().parse() {
            headers.append(header::SET_COOKIE, value);
        }
    }
}

pub fn read_session(jar: &CookieJar, key: &Key) -> Option<(Uuid, i64)> {
    let cookie = jar.signed(key).get(SESSION_COOKIE)?;
    let (id, exp) = cookie.value().rsplit_once('.')?;
    Some((Uuid::parse_str(id).ok()?, exp.parse().ok()?))
}

pub fn jar_from_request(headers: &axum::http::HeaderMap) -> CookieJar {
    let mut jar = CookieJar::new();
    let Some(value) = headers.get(header::COOKIE).and_then(|value| value.to_str().ok()) else {
        return jar;
    };
    for part in value.split(';') {
        if let Ok(cookie) = Cookie::parse(part.trim().to_owned()) {
            jar.add_original(cookie);
        }
    }
    jar
}

fn write_session(jar: &mut CookieJar, key: &Key, user_id: Uuid, exp: i64, secure: bool) {
    let mut builder = Cookie::build((SESSION_COOKIE, format!("{user_id}.{exp}")))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/admin")
        .max_age(cookie::time::Duration::seconds(SESSION_SECONDS));
    if secure {
        builder = builder.secure(true);
    }
    jar.signed_mut(key).add(builder.build());
}

async fn load_user(state: &AppState, id: Uuid) -> Result<Option<CurrentUser>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, display_name, is_active, role FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.and_then(|row| {
        if !row.is_active {
            return None;
        }
        Some(CurrentUser {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            role: Role::parse(&row.role),
        })
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn logout_emits_a_removal_cookie() {
        let key = cookie_key("test-secret");
        let mut issued = CookieJar::new();
        write_session(&mut issued, &key, Uuid::nil(), Utc::now().timestamp() + 100, false);
        let set = issued.delta().next().expect("session cookie");
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, format!("{}={}", set.name(), set.value()).parse().unwrap());

        let cleared = logout_cookie(&headers, false);
        let removal = cleared.delta().next().expect("removal cookie");
        assert_eq!(removal.name(), SESSION_COOKIE);
        assert_eq!(removal.path(), Some("/admin"));
        assert_eq!(removal.max_age(), Some(cookie::time::Duration::seconds(0)));
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
