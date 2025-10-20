use crate::{
    auth::SessionStore,
    dataset_client::DatasetManagerClient,
    db::Database, validators::strategy_validator::StrategyValidator,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub session_store: SessionStore,
    pub dataset_manager: DatasetManagerClient,
    pub strat_validator: StrategyValidator,
}
