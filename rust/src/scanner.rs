use crate::{Error, Location};

/// Why scanning stopped before a value was complete.
pub(crate) enum Stop {
    /// The end of the input was reached.
    Eof,
    /// The input is not valid LWON.
    Error(Error),
}

impl From<Error> for Stop {
    fn from(e: Error) -> Stop {
        Stop::Error(e)
    }
}

/// The result of a scanning operation that may stop early.
pub(crate) type Scan<T> = Result<T, Stop>;

/// Scans the characters of a string, tracking the location of each. Line
/// terminators are "\n", "\r\n", and "\r"; "\r\n" is a single terminator.
/// Positions are indices of characters in the input.
pub(crate) struct Scanner {
    chars: Vec<char>,
    name: String,
    pos: usize,
    locations: Vec<Location>,
}

impl Scanner {
    /// A scanner at the beginning of text. name identifies text in errors.
    pub fn new(text: &str, name: String) -> Scanner {
        let chars: Vec<char> = text.chars().collect();
        let mut locations = Vec::with_capacity(chars.len());
        let (mut line, mut column, mut after_cr) = (1, 0, false);
        for &c in &chars {
            if after_cr && c == '\n' {
                // the second character of "\r\n"
            } else if c == '\n' || c == '\r' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
            after_cr = c == '\r';
            locations.push(Location { line, column });
        }
        Scanner {
            chars,
            name,
            pos: 0,
            locations,
        }
    }

    /// The next character, if any.
    pub fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Whether there is a next character.
    pub fn has_next(&self) -> bool {
        self.pos < self.chars.len()
    }

    /// Scan past and return the next character. Stops with Eof if there is none.
    pub fn next(&mut self) -> Scan<char> {
        let c = self.peek().ok_or(Stop::Eof)?;
        self.pos += 1;
        Ok(c)
    }

    /// The current position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// The current position. Stops with Eof if there is no next character.
    pub fn here(&self) -> Scan<usize> {
        if self.has_next() {
            Ok(self.pos)
        } else {
            Err(Stop::Eof)
        }
    }

    /// Return to position pos.
    pub fn reset(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// The location of the character at pos.
    /// Requires: pos is the position of a character.
    pub fn location(&self, pos: usize) -> Location {
        self.locations[pos]
    }

    /// The column of the next character, or at the end of the input, the
    /// column of the last character (0 if the input is empty).
    pub fn column(&self) -> usize {
        match self.locations.get(self.pos).or(self.locations.last()) {
            Some(loc) => loc.column,
            None => 0,
        }
    }

    /// If the input is at a line terminator, scan past it and return true.
    /// Otherwise, return false.
    pub fn newline(&mut self) -> bool {
        match self.peek() {
            Some('\n') => self.pos += 1,
            Some('\r') => {
                self.pos += 1;
                if self.peek() == Some('\n') {
                    self.pos += 1;
                }
            }
            _ => return false,
        }
        true
    }

    /// An error with the given message at the character at pos.
    /// Requires: pos is the position of a character.
    pub fn error(&self, pos: usize, message: impl Into<String>) -> Error {
        Error::new(
            message,
            Some(&self.name),
            Some(self.locations[pos]),
            Some(self.chars[pos]),
        )
    }

    /// An error with the given message and no location.
    pub fn error_without_location(&self, message: impl Into<String>) -> Error {
        Error::new(message, Some(&self.name), None, None)
    }
}
