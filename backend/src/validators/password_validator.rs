use crate::errors::AppError;

pub fn validate_password(password: &String) -> Result<(), AppError> {
    if password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    if password.len() < 12 {
        return Err(AppError::BadRequest(
            "Password must be at least 12 characters long".to_string(),
        ));
    }

    if password.len() > 128 {
        return Err(AppError::BadRequest(
            "Password cannot be longer than 128 characters".to_string(),
        ));
    }

    let has_lower   = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper   = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit   = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| r"!@#$%^&*()_-+=[]{};:,.<>?/~\|".contains(c));

    let categories = [has_lower, has_upper, has_digit, has_special]
        .iter()
        .filter(|&&b| b)
        .count();

    if categories < 3 {
        return Err(AppError::BadRequest(
                "Password must contain at least 3 of these 4 : lowercase, uppercase, digit, special character".to_string()));
    }

    if password.chars()
        .collect::<Vec<_>>()
        .windows(4)
        .any(|w| w.iter().all(|&c| c == w[0]))
    {
        return Err(AppError::BadRequest(
                "Password cannot contain 4 identical characters in a row".to_string()));
    }

    if password.trim() != password {
        return Err(AppError::BadRequest(
                "Password cannot start or end with whitespace".to_string()));
    }

    // TODO: blacklist common passwords
    Ok(())
}

