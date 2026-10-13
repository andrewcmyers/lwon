use std::fmt;

use crate::Location;

/// An error indicating that input is not valid LWON, that an LWON value
/// cannot be deserialized into the requested type, or that data cannot be
/// serialized as LWON.
///
/// It is displayed as `"name", line L, char C (c): message`, where `name`
/// identifies the input and `c` is the character at the error. Parts that
/// the error does not have are omitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    message: String,
    name: Option<String>,
    location: Option<Location>,
    found: Option<char>,
}

impl Error {
    pub(crate) fn new(
        message: impl Into<String>,
        name: Option<&str>,
        location: Option<Location>,
        found: Option<char>,
    ) -> Error {
        Error {
            message: message.into(),
            name: name.map(str::to_string),
            location,
            found,
        }
    }

    /// This error, with the given location if it has none.
    #[cfg(feature = "serde")]
    pub(crate) fn at(mut self, location: Option<Location>) -> Error {
        self.location = self.location.or(location);
        self
    }

    /// This error, with the given name if it has none.
    #[cfg(feature = "serde")]
    pub(crate) fn named(mut self, name: &str) -> Error {
        self.name.get_or_insert_with(|| name.to_string());
        self
    }

    /// A description of the error, without its location.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The name identifying the input that contains the error, if known.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The location of the error, if known.
    pub fn location(&self) -> Option<Location> {
        self.location
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(name) = &self.name {
            parts.push(format!("\"{name}\""));
        }
        if let Some(loc) = self.location {
            let mut part = format!("line {}, char {}", loc.line, loc.column);
            if let Some(c) = self.found {
                part.push_str(&format!(" ({})", display_char(c)));
            }
            parts.push(part);
        }
        if !parts.is_empty() {
            write!(f, "{}: ", parts.join(", "))?;
        }
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "serde")]
impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Error {
        Error::new(message.to_string(), None, None, None)
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Error {
        Error::new(message.to_string(), None, None, None)
    }
}

/// c, made visible if it is a control character.
fn display_char(c: char) -> String {
    match c {
        '\n' => "\\n".to_string(),
        '\t' => "\\t".to_string(),
        '\r' => "\\r".to_string(),
        '\u{8}' => "\\b".to_string(),
        c if (c as u32) < 0x20 => format!("\\x{{{:02x}}}", c as u32),
        c => c.to_string(),
    }
}
