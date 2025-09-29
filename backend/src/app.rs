use crate::{auth::SessionStore, db::Database};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub session_store: SessionStore,
}
