use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, Duration, UNIX_EPOCH};

use crate::models::models_api::User;

const SECRET: &[u8] = b"super-secret-key-change-me"; // store in env var in prod

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: User, // user_id
    exp: usize, // expiration
}

pub fn generate_jwt(user: User) -> String {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(60 * 60)) // 1h
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims { sub: user, exp: expiration };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET)).unwrap()
}

pub async fn validate_jwt(token: &str) -> Option<User>{
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::default(), // What is the default ?
    ).ok()?;

    Some(decoded.claims.sub)
}
