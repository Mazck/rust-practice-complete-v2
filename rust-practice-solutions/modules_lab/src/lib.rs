pub mod domain;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("name is empty")]
    EmptyName,
    #[error("invalid email")]
    InvalidEmail,
}

pub fn create_user(name: &str, email: &str) -> Result<domain::User, DomainError> {
    if name.trim().is_empty() {
        return Err(DomainError::EmptyName);
    }
    let email = domain::Email::parse(email).map_err(|_| DomainError::InvalidEmail)?;
    Ok(domain::User::new(name, email, domain::Role::User))
}

/// Adds two integers.
///
/// ```
/// assert_eq!(modules_lab::add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_name() {
        assert_eq!(
            create_user("", "a@example.com"),
            Err(DomainError::EmptyName)
        );
    }

    #[test]
    fn creates_valid_user() {
        let user = create_user("An", "an@example.com").unwrap();
        assert_eq!(user.name(), "An");
    }
}
