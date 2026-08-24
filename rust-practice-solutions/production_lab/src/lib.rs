use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

#[derive(Debug, thiserror::Error)]
pub enum ProductionError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("password hash error: {0}")]
    PasswordHash(String),
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Telemetry {
    pub device_id: String,
    pub message_id: String,
    pub value: f64,
    pub unit: String,
}

impl Telemetry {
    pub fn validate(&self) -> Result<(), ProductionError> {
        if self.device_id.trim().is_empty() {
            return Err(ProductionError::Validation("device_id is empty".into()));
        }
        if self.message_id.trim().is_empty() {
            return Err(ProductionError::Validation("message_id is empty".into()));
        }
        if !self.value.is_finite() {
            return Err(ProductionError::Validation("value is not finite".into()));
        }
        if !matches!(self.unit.as_str(), "C" | "%" | "Pa") {
            return Err(ProductionError::Validation("unsupported unit".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub iss: String,
    pub exp: usize,
}

pub fn hash_password(password: &[u8]) -> Result<String, ProductionError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password, &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| ProductionError::PasswordHash(error.to_string()))
}

pub fn verify_password(password: &[u8], encoded: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(encoded) else {
        return false;
    };
    Argon2::default().verify_password(password, &parsed).is_ok()
}

pub fn issue_token(claims: &Claims, secret: &[u8]) -> Result<String, ProductionError> {
    Ok(encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret),
    )?)
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, ProductionError> {
    let mut validation = Validation::default();
    validation.set_issuer(&["device-gateway"]);
    Ok(decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?.claims)
}

pub async fn open_database(url: &str) -> Result<SqlitePool, ProductionError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect(url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn record_with_outbox(
    pool: &SqlitePool,
    value: &Telemetry,
    topic: &str,
    payload: &[u8],
) -> Result<bool, ProductionError> {
    value.validate()?;
    let mut tx = pool.begin().await?;

    let inserted = sqlx::query("INSERT OR IGNORE INTO processed_messages(message_id) VALUES (?)")
        .bind(&value.message_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if inserted == 0 {
        tx.rollback().await?;
        return Ok(false);
    }

    sqlx::query("INSERT INTO telemetry(device_id, message_id, value, unit) VALUES (?, ?, ?, ?)")
        .bind(&value.device_id)
        .bind(&value.message_id)
        .bind(value.value)
        .bind(&value.unit)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO outbox(message_id, topic, payload) VALUES (?, ?, ?)")
        .bind(&value.message_id)
        .bind(topic)
        .bind(payload)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_round_trip() {
        let encoded = hash_password(b"correct horse battery staple").unwrap();
        assert!(verify_password(b"correct horse battery staple", &encoded));
        assert!(!verify_password(b"wrong", &encoded));
    }

    #[test]
    fn jwt_round_trip() {
        let claims = Claims {
            sub: "user-1".into(),
            role: "admin".into(),
            iss: "device-gateway".into(),
            exp: 4_000_000_000,
        };
        let token = issue_token(&claims, b"test-secret").unwrap();
        let decoded = verify_token(&token, b"test-secret").unwrap();
        assert_eq!(decoded.sub, "user-1");
    }

    #[tokio::test]
    async fn duplicate_message_is_ignored() {
        let pool = open_database("sqlite::memory:").await.unwrap();
        let value = Telemetry {
            device_id: "d-1".into(),
            message_id: "m-1".into(),
            value: 21.5,
            unit: "C".into(),
        };
        assert!(
            record_with_outbox(&pool, &value, "topic", b"payload")
                .await
                .unwrap()
        );
        assert!(
            !record_with_outbox(&pool, &value, "topic", b"payload")
                .await
                .unwrap()
        );
    }
}
