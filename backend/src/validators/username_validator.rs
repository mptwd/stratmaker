use crate::errors::AppError;

pub fn validate_username(username: &String) -> Result<(), AppError> {
    if username.is_empty() {
        return Err(AppError::BadRequest("Username is required".to_string()));
    }

    if username.len() < 3 {
        return Err(AppError::BadRequest(
            "Username cannot be smaller than 3 characters".to_string(),
        ));
    }

    if username.len() > 25 {
        return Err(AppError::BadRequest(
            "Username cannot be greater than 25 characters".to_string(),
        ));
    }


    /*
     * At least 1 letter.
     * Starts with a letter or a digit.
     * 3-25 characters.
     * Only letters, digits, hyphen and underscores are allowed.
     * Hyphen and underscores have to be followed by a letter or a digit.
     */
    let chars: Vec<char> = username.chars().collect();

    // Must start with letter or digit
    if !chars[0].is_ascii_alphanumeric() {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // At least 1 letter
    if !chars.iter().any(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // Allowed characters only
    if !chars
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
    {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // Hyphen/underscore must be followed by a letter/digit
    for w in chars.windows(2) {
        if (w[0] == '-' || w[0] == '_') && !w[1].is_ascii_alphanumeric() {
            return Err(AppError::BadRequest(
                "Must be a valid username".to_string(),
            ));
        }
    }

    if chars.last() == Some(&'-') || chars.last() == Some(&'_') {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    Ok(())
}
