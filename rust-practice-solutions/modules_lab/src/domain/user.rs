use super::{Email, Role};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    name: String,
    email: Email,
    role: Role,
}

impl User {
    pub fn new(name: impl Into<String>, email: Email, role: Role) -> Self {
        Self {
            name: name.into(),
            email,
            role,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        self.email.as_str()
    }

    pub fn role(&self) -> Role {
        self.role
    }
}
