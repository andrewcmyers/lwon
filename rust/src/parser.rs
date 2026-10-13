use crate::scanner::{Scan, Scanner, Stop};
use crate::unparse::{LINE_SEPARATOR, escape};
use crate::{Array, Dictionary, Error, Text, Value};

const UNEXPECTED_EOF: &str = "Unexpected end of input";
const RESERVED_CHARS: &str = "{}[]:";
const BLANKS: &str = " \t";
const LINE_TERMINATORS: &str = "\n\r\u{c}";
const WHITESPACE: &str = " \t\n\r\u{c}";

/// Whether c is a character (not the end of the input) in chars.
fn is_in(c: Option<char>, chars: &str) -> bool {
    c.is_some_and(|c| chars.contains(c))
}

/// Whether c is the first character of a line terminator.
fn is_newline(c: Option<char>) -> bool {
    matches!(c, Some('\n' | '\r'))
}

/// The text denoted by the escape sequence consisting of a backslash and c,
/// if c is one of the characters that follow a backslash without more data.
fn simple_escape(c: char) -> Option<&'static str> {
    Some(match c {
        'r' => "\r",
        'n' => "\n",
        ' ' => " ",
        'f' => "\u{c}",
        'b' => "\u{8}",
        't' => "\t",
        '\\' => "\\",
        '/' => "/",
        '"' => "\"",
        '\'' => "'",
        '}' => "}",
        ']' => "]",
        '#' => "#",
        ':' => ":",
        '\n' => "",
        _ => return None,
    })
}

/// c, escaped as in an LWON string except for "/", for use in error messages.
fn unparse_char(c: char) -> String {
    let mut b = String::new();
    match c {
        '/' => b.push(c),
        _ => escape(c.encode_utf8(&mut [0; 4]), &mut b),
    }
    b
}

/// Parses a sequence of LWON values from a string.
///
/// Each parsing method returns `Ok(None)` if there are no more values in the
/// input, and an [`Error`] if the input is not valid LWON. After an error,
/// the parser should not be used further.
pub struct Parser {
    s: Scanner,
}

impl Parser {
    /// A parser for the values in text. name identifies text in errors; for
    /// example, it may be a file name.
    pub fn new(text: &str, name: impl Into<String>) -> Parser {
        Parser {
            s: Scanner::new(text, name.into()),
        }
    }

    /// Parse the next value.
    pub fn parse(&mut self) -> Result<Option<Value>, Error> {
        let r = self.skip_comments().and_then(|()| self.parse_value(""));
        finish(r)
    }

    /// Parse the next value as an implicit array, which is not enclosed in
    /// brackets.
    pub fn parse_implicit_array(&mut self) -> Result<Option<Array>, Error> {
        let r = self.skip_comments().and_then(|()| {
            let result = self.parse_array(self.s.here()?)?;
            self.reject_unmatched(']')?;
            Ok(result)
        });
        finish(r)
    }

    /// Parse the next value as an implicit dictionary, which is not enclosed in
    /// braces.
    pub fn parse_implicit_dictionary(&mut self) -> Result<Option<Dictionary>, Error> {
        let r = self.skip_comments().and_then(|()| {
            let result = self.parse_dictionary(self.s.here()?)?;
            self.reject_unmatched('}')?;
            Ok(result)
        });
        finish(r)
    }

    /// Parse the input, which must contain exactly one value.
    pub(crate) fn parse_only(mut self) -> Result<Value, Error> {
        let Some(value) = self.parse()? else {
            return Err(self.s.error_without_location("No LWON value"));
        };
        self.skip_whitespace();
        if self.s.has_next() {
            return Err(self
                .s
                .error(self.s.pos(), "Unexpected data after the value"));
        }
        Ok(value)
    }

    fn parse_value(&mut self, delimiters: &str) -> Scan<Value> {
        self.skip_whitespace();
        let first = self.s.here()?;
        let location = self.s.location(first);
        match self.s.next()? {
            '{' => {
                let result = self.parse_dictionary(first)?;
                self.expect_closing('}', first)?;
                Ok(result.into())
            }
            '[' => {
                let result = self.parse_array(first)?;
                self.expect_closing(']', first)?;
                Ok(result.into())
            }
            '"' => {
                let result = self.parse_text(true, delimiters, first)?;
                self.expect('"')?;
                Ok(Text::at(result, location).into())
            }
            c if RESERVED_CHARS.contains(c) => Err(self.error(
                first,
                format!("Unexpected reserved character {}", unparse_char(c)),
            )),
            _ => {
                self.s.reset(first);
                Ok(Text::at(self.parse_text(false, delimiters, first)?, location).into())
            }
        }
    }

    fn error(&self, pos: usize, message: impl Into<String>) -> Stop {
        Stop::Error(self.s.error(pos, message))
    }

    /// r, but with the end of the input reported as an unexpected end of input
    /// at pos.
    fn eof_is_error<T>(&self, r: Scan<T>, pos: usize) -> Scan<T> {
        match r {
            Err(Stop::Eof) => Err(self.error(pos, UNEXPECTED_EOF)),
            r => r,
        }
    }

    fn expect(&mut self, expected: char) -> Scan<usize> {
        let here = self.s.here()?;
        if self.s.next()? != expected {
            return Err(self.error(here, format!("Expected {}", unparse_char(expected))));
        }
        Ok(here)
    }

    fn expect_closing(&mut self, closing: char, opening: usize) -> Scan<()> {
        match self.expect(closing) {
            Err(Stop::Eof) => Err(self.error(
                opening,
                format!("Missing closing {}", unparse_char(closing)),
            )),
            r => r.map(|_| ()),
        }
    }

    fn reject_unmatched(&self, closing: char) -> Scan<()> {
        if self.s.peek() == Some(closing) {
            return Err(self.error(self.s.pos(), format!("Unmatched {}", unparse_char(closing))));
        }
        Ok(())
    }

    fn parse_dictionary(&mut self, start: usize) -> Scan<Dictionary> {
        let mut result = Dictionary::at(self.s.location(start));
        let r = self.parse_entries(&mut result);
        self.eof_is_error(r, start)?;
        Ok(result)
    }

    fn parse_entries(&mut self, result: &mut Dictionary) -> Scan<()> {
        loop {
            self.skip_whitespace();
            if matches!(self.s.peek(), None | Some('}')) {
                return Ok(());
            }
            let here = self.s.here()?;
            let key = self.parse_key(here)?;
            if is_newline(self.s.peek()) {
                return Err(self.error(here, "keys must have a non-newline delimiter"));
            }
            if self.s.peek() == Some(':') {
                self.s.next()?;
                self.skip_whitespace();
            }
            let value = self.parse_value("}")?;
            result.push(key, value);
            self.skip_whitespace();
            if self.s.peek() == Some(',') {
                self.s.next()?;
            }
        }
    }

    fn parse_key(&mut self, start: usize) -> Scan<String> {
        if self.s.peek() == Some('"') {
            self.s.next()?;
            let key = self.parse_text(true, ":", start)?;
            self.expect('"')?;
            self.skip_blanks();
            Ok(key)
        } else {
            self.parse_text(false, ":", start)
        }
    }

    /// Parse a long string (if multiline) after its opening quotation mark, or
    /// a short string ending before one of the delimiters. The string begins
    /// at start for error reporting.
    fn parse_text(&mut self, multiline: bool, delimiters: &str, start: usize) -> Scan<String> {
        let r = self.parse_text_contents(multiline, delimiters, start);
        self.eof_is_error(r, start)
    }

    fn parse_text_contents(
        &mut self,
        multiline: bool,
        delimiters: &str,
        start: usize,
    ) -> Scan<String> {
        let mut b = String::new();
        let mut ws = String::new(); // blanks that are kept only if more text follows
        let mut left_column = self.s.column();
        if multiline {
            let saved = self.s.pos();
            self.skip_blanks();
            if is_newline(self.s.peek()) {
                self.s.newline();
                left_column = self.skip_leading_space(left_column);
            } else {
                self.s.reset(saved);
            }
        }
        if !multiline && let Some(c) = self.s.peek().filter(|&c| RESERVED_CHARS.contains(c)) {
            return Err(self.error(
                start,
                format!("Unexpected reserved character {c} at start of short string"),
            ));
        }
        loop {
            let Some(ch) = self.s.peek() else {
                if multiline {
                    return Err(self.error(start, UNEXPECTED_EOF));
                }
                return Ok(b);
            };
            match ch {
                '\\' => {
                    if !multiline {
                        b.push_str(&ws);
                        ws.clear();
                    }
                    self.parse_escape_sequence(&mut b)?;
                }
                '"' if multiline => {
                    let saved = self.s.pos();
                    self.s.next()?;
                    if self.s.peek() == Some('"') {
                        b.push(self.s.next()?); // "" stands for "
                    } else {
                        self.s.reset(saved);
                        return Ok(b);
                    }
                }
                '"' | '{' | '[' => {
                    if !multiline && !ws.is_empty() {
                        return Ok(b);
                    }
                    b.push(self.s.next()?);
                }
                ' ' => {
                    let c = self.s.next()?;
                    if multiline { b.push(c) } else { ws.push(c) }
                }
                '\n' | '\r' => {
                    if !multiline {
                        return Ok(b);
                    }
                    self.s.newline();
                    self.skip_comments()?;
                    b.push_str(LINE_SEPARATOR);
                    left_column = self.skip_leading_space(left_column);
                }
                _ => {
                    if !multiline && delimiters.contains(ch) {
                        return Ok(b);
                    }
                    b.push_str(&ws);
                    ws.clear();
                    b.push(self.s.next()?);
                }
            }
        }
    }

    /// Skip past any comment lines to the beginning of the next line that is
    /// not a comment. Stops with Eof if the end of the input is reached.
    /// Requires: the scanner is at the beginning of a line.
    fn skip_comments(&mut self) -> Scan<()> {
        loop {
            let saved = self.s.pos();
            loop {
                match self.s.peek() {
                    None => {
                        self.s.reset(saved);
                        return Err(Stop::Eof);
                    }
                    Some('\n' | '\r') => {
                        self.s.reset(saved); // blank line
                        return Ok(());
                    }
                    Some('#') => {
                        while self.s.peek().is_some_and(|c| !LINE_TERMINATORS.contains(c)) {
                            self.s.next()?;
                        }
                        if !self.s.newline() {
                            self.s.next()?; // form feed
                        }
                        break;
                    }
                    Some(c) if BLANKS.contains(c) => {
                        self.s.next()?;
                    }
                    Some(_) => {
                        self.s.reset(saved); // not a comment line
                        return Ok(());
                    }
                }
            }
        }
    }

    /// Skip spaces before left_column, and return the resulting column.
    fn skip_leading_space(&mut self, left_column: usize) -> usize {
        while self.s.peek() == Some(' ') && self.s.column() < left_column {
            self.s.reset(self.s.pos() + 1);
        }
        self.s.column()
    }

    /// Parse an escape sequence and append the text it denotes to b.
    fn parse_escape_sequence(&mut self, b: &mut String) -> Scan<()> {
        let here = self.expect('\\')?;
        let first = self.s.next()?;
        if let Some(text) = simple_escape(first) {
            b.push_str(text);
            return Ok(());
        }
        match first {
            '\r' => {
                if self.s.peek() == Some('\n') {
                    self.s.next()?;
                }
            }
            'u' => b.push(self.parse_unicode_escape(here)?),
            _ => return Err(self.error(here, "Illegal escape sequence")),
        }
        Ok(())
    }

    /// Parse the rest of a Unicode escape sequence that begins at here, after
    /// its `u`, and return the character it denotes. An escape sequence for a
    /// high surrogate must be immediately followed by one for a low surrogate,
    /// and together they denote one supplementary character.
    fn parse_unicode_escape(&mut self, here: usize) -> Scan<char> {
        let braced = self.s.peek() == Some('{');
        let code = self.parse_code_point(braced, here)?;
        if !braced && (0xD800..0xDC00).contains(&code) {
            let low = self.parse_low_surrogate(here)?;
            return Ok(char::from_u32(0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00)).unwrap());
        }
        if !braced && (0xDC00..0xE000).contains(&code) {
            return Err(self.error(here, "Unpaired surrogate in character escape"));
        }
        char::from_u32(code)
            .ok_or_else(|| self.error(here, "Invalid codepoint in character escape"))
    }

    /// Parse the hex digits of a Unicode escape sequence that begins at here:
    /// exactly four digits, or if braced, one to six digits enclosed in braces.
    /// Return the number they denote.
    fn parse_code_point(&mut self, braced: bool, here: usize) -> Scan<u32> {
        if braced {
            self.expect('{')?;
        }
        let max_digits = if braced { 6 } else { 4 };
        let mut code = 0;
        let mut digits = 0;
        while digits < max_digits
            && let Some(d) = self.s.peek().and_then(|c| c.to_digit(16))
        {
            self.s.next()?;
            code = code * 16 + d;
            digits += 1;
        }
        if digits == 0 || !braced && digits < 4 {
            return Err(self.error(here, "Expected hex digits in character escape"));
        }
        if braced {
            self.expect('}')?;
        }
        Ok(code)
    }

    /// Parse a four-digit Unicode escape sequence for a low surrogate, which
    /// completes the surrogate pair whose escape sequence begins at here, and
    /// return the surrogate.
    fn parse_low_surrogate(&mut self, here: usize) -> Scan<u32> {
        if self.s.peek() == Some('\\') {
            self.s.next()?;
            if self.s.peek() == Some('u') {
                self.s.next()?;
                if self.s.peek() != Some('{') {
                    let code = self.parse_code_point(false, here)?;
                    if (0xDC00..0xE000).contains(&code) {
                        return Ok(code);
                    }
                }
            }
        }
        Err(self.error(here, "Unpaired surrogate in character escape"))
    }

    fn skip_blanks(&mut self) {
        while is_in(self.s.peek(), BLANKS) {
            self.s.reset(self.s.pos() + 1);
        }
    }

    /// Skip past whitespace, including line terminators, and comments.
    fn skip_whitespace(&mut self) {
        while is_in(self.s.peek(), WHITESPACE) {
            if self.s.newline() {
                if self.skip_comments().is_err() {
                    return;
                }
            } else {
                self.s.reset(self.s.pos() + 1);
            }
        }
    }

    fn parse_array(&mut self, start: usize) -> Scan<Array> {
        let mut entries = Vec::new();
        let mut here = start;
        match self.parse_elements(&mut entries, &mut here) {
            Ok(()) => Ok(build_array(entries, self.s.location(start))),
            Err(Stop::Eof) => Err(self.error(here, UNEXPECTED_EOF)),
            Err(e) => Err(e),
        }
    }

    /// Parse the elements of an array, adding each to entries along with its
    /// indices, which omit leading zeros. here is kept at the position to
    /// report if the input ends unexpectedly.
    fn parse_elements(
        &mut self,
        entries: &mut Vec<(Vec<usize>, Value)>,
        here: &mut usize,
    ) -> Scan<()> {
        let mut indices: Vec<usize> = Vec::new();
        loop {
            self.skip_whitespace();
            if matches!(self.s.peek(), None | Some(']')) {
                return Ok(());
            }
            *here = self.s.pos();
            entries.push((indices.clone(), self.parse_value(",]")?));
            self.skip_blanks();
            match self.s.peek() {
                Some(',') => {
                    match indices.last_mut() {
                        Some(last) => *last += 1,
                        None => indices = vec![1],
                    }
                    self.s.next()?;
                    self.skip_whitespace();
                }
                None | Some(']') => return Ok(()),
                Some('\n' | '\r') => {
                    let mut dimout = 0;
                    while self.s.has_next() {
                        *here = self.s.pos();
                        if !self.s.newline() || self.skip_comments().is_err() {
                            break;
                        }
                        dimout += 1;
                    }
                    // The dimension whose index increases, counting from the last
                    let index = if dimout + 1 > indices.len() {
                        indices = vec![0; dimout + 1];
                        0
                    } else {
                        let index = indices.len() - 1 - dimout;
                        indices[index + 1..].fill(0);
                        index
                    };
                    indices[index] += 1;
                }
                Some(c) => {
                    return Err(self.error(
                        self.s.pos(),
                        format!("Unexpected character {}", unparse_char(c)),
                    ));
                }
            }
        }
    }
}

/// The result of a parsing method, given the result r of parsing.
fn finish<T>(r: Scan<T>) -> Result<Option<T>, Error> {
    match r {
        Ok(value) => Ok(Some(value)),
        Err(Stop::Eof) => Ok(None),
        Err(Stop::Error(e)) => Err(e),
    }
}

/// The array beginning at location whose elements are given by entries, a
/// list of (indices, value) pairs. Indices shorter than the number of
/// dimensions are padded with leading zeros. Positions with no entry hold
/// empty text.
fn build_array(entries: Vec<(Vec<usize>, Value)>, location: crate::Location) -> Array {
    let ndim = entries.iter().map(|(ix, _)| ix.len()).max().unwrap_or(0);
    if ndim == 0 {
        let elements: Vec<Value> = entries.into_iter().map(|(_, v)| v).collect();
        return Array::at(vec![elements.len()], elements, Some(location));
    }
    let mut dimensions = vec![0; ndim];
    for (ix, _) in &entries {
        let offset = ndim - ix.len();
        for (i, &n) in ix.iter().enumerate() {
            dimensions[i + offset] = dimensions[i + offset].max(n + 1);
        }
    }
    let size = dimensions.iter().product();
    let mut elements = vec![Value::Text(Text::at(String::new(), location)); size];
    for (ix, value) in entries {
        let offset = ndim - ix.len();
        let i = ix
            .iter()
            .enumerate()
            .fold(0, |i, (j, &n)| i * dimensions[j + offset] + n);
        elements[i] = value;
    }
    Array::at(dimensions, elements, Some(location))
}
