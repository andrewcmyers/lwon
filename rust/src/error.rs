use std::fmt;

use crate::Location;

/// An error indicating that input is not valid LWON.
///
/// It is displayed as `"name", line L, char C (c): message`, where `name`
/// identifies the input and `c` is the character at the error, or as
/// `"name": message` if the error has no location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    message: String,
    name: String,
    location: Option<Location>,
    found: Option<char>,
}

impl Error {
    pub(crate) fn new(
        message: impl Into<String>,
        name: &str,
        location: Option<Location>,
        found: Option<char>,
    ) -> Error {
        Error {
            message: message.into(),
            name: name.to_string(),
            location,
            found,
        }
    }

    /// A description of the error, without its location.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The name identifying the input that contains the error.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The location of the error, if it has one.
    pub fn location(&self) -> Option<Location> {
        self.location
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.location {
            Some(loc) => {
                write!(
                    f,
                    "\"{}\", line {}, char {}",
                    self.name, loc.line, loc.column
                )?;
                if let Some(c) = self.found {
                    write!(f, " ({})", display_char(c))?;
                }
                write!(f, ": {}", self.message)
            }
            None => write!(f, "\"{}\": {}", self.name, self.message),
        }
    }
}

impl std::error::Error for Error {}

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
