use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::errors::AppError;

// Extractor for authenticated user ID.
// This will automatically be populated by the auth middleware.
// 
// # Example
// ```
// async fn my_handler(
//     State(state): State<AppState>,
//     AuthenticatedUser(user_id): AuthenticatedUser,
// ) -> Result<Json<Value>, AppError> {
//     // user_id is guaranteed to exist here
//     Ok(Json(json!({"user_id": user_id})))
// }
// ```

#[derive(Debug, Clone, Copy)]
pub struct AuthenticatedUser(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Uuid>()
            .copied()
            .map(AuthenticatedUser)
            .ok_or(AppError::Unauthorized)
    }
}

impl AuthenticatedUser {
    /// Get the user ID
    pub fn id(&self) -> Uuid {
        self.0
    }

    /// Consume the extractor and return the inner UUID
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

// Implement Deref for more ergonomic usage
impl std::ops::Deref for AuthenticatedUser {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
