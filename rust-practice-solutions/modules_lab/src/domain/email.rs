#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.contains('@') && !value.starts_with('@') && !value.ends_with('@') {
            Ok(Self(value))
        } else {
            Err("invalid email")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
