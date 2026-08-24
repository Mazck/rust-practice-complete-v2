pub mod api;
pub mod protocol;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Temperature {
    pub device_id: String,
    pub value: f32,
    pub unit: String,
    pub message_id: String,
}

impl Temperature {
    pub fn validate(&self) -> Result<(), NetworkError> {
        if self.device_id.trim().is_empty() {
            return Err(NetworkError::InvalidInput("device_id is empty".into()));
        }
        if self.message_id.trim().is_empty() {
            return Err(NetworkError::InvalidInput("message_id is empty".into()));
        }
        if !self.value.is_finite() {
            return Err(NetworkError::InvalidInput("value is not finite".into()));
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
}

pub fn parse_temperature(payload: &[u8]) -> Result<Temperature, NetworkError> {
    let value: Temperature = serde_json::from_slice(payload)?;
    value.validate()?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_validates_temperature() {
        let value =
            parse_temperature(br#"{"device_id":"d-1","value":21.5,"unit":"C","message_id":"m-1"}"#)
                .unwrap();
        assert_eq!(value.device_id, "d-1");
    }

    #[test]
    fn rejects_empty_device() {
        let result =
            parse_temperature(br#"{"device_id":"","value":21.5,"unit":"C","message_id":"m-1"}"#);
        assert!(matches!(result, Err(NetworkError::InvalidInput(_))));
    }
}
