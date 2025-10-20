use regex::Regex;
use crate::errors::AppError;

pub fn validate_email(email: &String) -> Result<(), AppError> {
    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }

    if email.len() > 255 {
        return Err(AppError::BadRequest(
            "Email cannot be greater than 255 characters".to_string(),
        ));
    }

    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !re.is_match(&email) {
        return Err(AppError::BadRequest("Must be a valid email".to_string()));
    }
    Ok(())
}
