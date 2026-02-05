use aws_sdk_s3::Client;

use crate::{
    auth::SessionStore, dataset_client::DatasetManagerClient, db::Database, s3_manager,
    s3_manager::S3manager, validators::strategy_validator::StrategyValidator,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub session_store: SessionStore,
    pub dataset_manager: DatasetManagerClient,
    pub strat_validator: StrategyValidator,
    pub s3: S3manager,
}
