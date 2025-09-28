use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use axum_extra::extract::CookieJar;

use crate::{errors::AppError, AppState};



pub async fn auth_middleware(
    State(state): State<AppState>, // optional if you want DB access
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {

    if let Some(session_cookie) = jar.get("session_id") {
        let session_id = session_cookie.value();

        // Look it up in Redis
        if let Some(session) = state.session_store.get_session(session_id).await? {
            // Extend session on each request
            state.session_store.extend_session(session_id).await?;

            req.extensions_mut().insert(session.user_id); // Adding to the request only the id for now.
            return Ok(next.run(req).await);
        }

        return Err(AppError::Unauthorized);
    }

    // Here, we continue if no session_id cookie was present.
    // But is that correct ? Should we not stop that ?
    Ok(next.run(req).await)

}
