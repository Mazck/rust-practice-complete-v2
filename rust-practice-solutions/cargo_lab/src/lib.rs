include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[doc(hidden)]
pub fn __format_internal(value: &str) -> String {
    format!("<{value}>")
}

#[macro_export]
macro_rules! format_public {
    ($value:expr) => {
        $crate::__format_internal($value)
    };
}

pub fn render(value: &str) -> String {
    #[cfg(feature = "pretty")]
    {
        format!("*** {value} ***")
    }

    #[cfg(all(not(feature = "pretty"), feature = "compact"))]
    {
        value.to_owned()
    }

    #[cfg(not(any(feature = "pretty", feature = "compact")))]
    {
        format!("[{value}]")
    }
}

/// Returns a stable answer.
///
/// ```
/// assert_eq!(cargo_lab::answer(), 42);
/// ```
pub fn answer() -> u32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_version_exists() {
        assert!(!APP_VERSION.is_empty());
    }

    #[test]
    fn macro_uses_crate_path() {
        assert_eq!(format_public!("ok"), "<ok>");
    }
}
