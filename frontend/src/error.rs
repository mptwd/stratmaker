use serde::Deserialize;

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
