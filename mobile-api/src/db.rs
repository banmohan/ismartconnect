use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{hash_password, KeyStore};
use crate::config::Config;

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn ensure_log_partitions(pool: &PgPool) -> anyhow::Result<()> {
    let start = Utc::now().date_naive().with_day(1).expect("day 1 exists");
    for offset in [-1, 0, 1] {
        let month = add_months(start, offset);
        if let Err(err) = sqlx::query("SELECT ensure_request_log_partition($1::date)")
            .bind(month)
            .execute(pool)
            .await
        {
            tracing::error!(?err, %month, "could not ensure request log partition");
        }
    }
    Ok(())
}

pub async fn seed_admin(pool: &PgPool, config: &Config) -> anyhow::Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(pool).await?;
    if count > 0 {
        return Ok(());
    }
    let (Some(username), Some(password)) = (&config.admin_username, &config.admin_password) else {
        tracing::warn!("no admin user seeded; set ADMIN_USERNAME and ADMIN_PASSWORD");
        return Ok(());
    };
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, display_name, is_active, role, created_at, updated_at)
         VALUES ($1, $2, $3, $4, true, 'admin', $5, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(username)
    .bind(hash_password(password)?)
    .bind("Administrator")
    .bind(now)
    .execute(pool)
    .await?;
    tracing::info!(%username, "seeded admin user");
    Ok(())
}

pub async fn load_keys(pool: &PgPool) -> anyhow::Result<KeyStore> {
    Ok(KeyStore::load(pool).await?)
}

fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let month0 = date.month0() as i32 + months;
    let year = date.year() + month0.div_euclid(12);
    let month = month0.rem_euclid(12) as u32 + 1;
    NaiveDate::from_ymd_opt(year, month, 1).expect("month is valid")
}
