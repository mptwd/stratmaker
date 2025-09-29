use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

const SESSION_TTL: u64 = 86400; // 24 hours in seconds

#[derive(Clone)]
pub struct SessionStore {
    redis: Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl SessionStore {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = Client::open(redis_url)?;
        
        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        
        Ok(Self { redis: client })
    }

    pub fn from_client(redis_client: Client) -> Self {
        Self { redis: redis_client }
    }

    pub async fn create_session(&self, user_id: Uuid) -> Result<String, AppError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(SESSION_TTL as i64);

        let session = Session {
            user_id,
            created_at: now,
            expires_at,
        };

        let session_json = serde_json::to_string(&session)?;
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        let _: () = conn.set_ex(&session_id, session_json, SESSION_TTL).await?;

        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>, AppError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        let session_data: Option<String> = conn.get(session_id).await?;
        
        match session_data {
            Some(data) => {
                let session: Session = serde_json::from_str(&data)?;
                
                // Check if session is expired
                if session.expires_at < Utc::now() {
                    // Clean up expired session
                    let _: () = conn.del(session_id).await?;
                    Ok(None)
                } else {
                    Ok(Some(session))
                }
            }
            None => Ok(None),
        }
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), AppError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: () = conn.del(session_id).await?;
        Ok(())
    }

    pub async fn extend_session(&self, session_id: &str) -> Result<(), AppError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: () = conn.expire(session_id, SESSION_TTL.try_into().unwrap()).await?;
        Ok(())
    }
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::PasswordHash)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| AppError::PasswordHash)?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}
