use crate::{Array, Dictionary, Value};

/// The standard newline sequence on the current operating system.
pub(crate) const LINE_SEPARATOR: &str = if cfg!(windows) { "\r\n" } else { "\n" };

const RESERVED_START: &str = "{}[]:\"|$+\\#";
const RESERVED_ANYWHERE: &str = "{}[]:\"\\";

/// Append the LWON text for value to b, with nested lines indented by indent
/// levels.
pub(crate) fn unparse(value: &Value, b: &mut String, indent: usize) {
    match value {
        Value::Text(t) => {
            b.push('"');
            escape(t, b);
            b.push('"');
        }
        Value::Array(a) => unparse_array(a, b, indent),
        Value::Dictionary(d) => unparse_dictionary(d, b, indent),
    }
}

/// Append to b the text of s as written between the quotation marks of an LWON
/// string. The characters `"`, `\`, `/`, and control characters are written
/// as escape sequences, as are characters outside ASCII: `\uXXXX` for those in
/// the Basic Multilingual Plane and `\u{XXXXX}` for supplementary characters.
pub(crate) fn escape(s: &str, b: &mut String) {
    for c in s.chars() {
        match c {
            '"' => b.push_str("\\\""),
            '\\' => b.push_str("\\\\"),
            '/' => b.push_str("\\/"),
            '\u{8}' => b.push_str("\\b"),
            '\n' => b.push_str("\\n"),
            '\t' => b.push_str("\\t"),
            '\u{c}' => b.push_str("\\f"),
            '\r' => b.push_str("\\r"),
            ' '..='\u{7f}' => b.push(c),
            '\u{80}'..='\u{ffff}' | '\0'..='\u{1f}' => b.push_str(&format!("\\u{:04X}", c as u32)),
            _ => b.push_str(&format!("\\u{{{:X}}}", c as u32)),
        }
    }
}

fn newline(b: &mut String, indent: usize) {
    b.push_str(LINE_SEPARATOR);
    b.push_str(&"  ".repeat(indent));
}

fn unparse_array(a: &Array, b: &mut String, indent: usize) {
    let dimensions = a.dimensions();
    let n = dimensions.len();
    b.push('[');
    if a.is_empty() {
        newline(b, indent);
        b.push(']');
        return;
    }
    let mut indices = vec![0; n];
    let mut newlines = 1;
    for element in a {
        if newlines > 0 {
            b.push_str(&LINE_SEPARATOR.repeat(newlines));
            b.push_str(&"  ".repeat(indent + 1));
        } else {
            b.push(' ');
        }
        unparse(element, b, indent + 1);
        // Advance indices to the next element, counting the dimensions that end.
        newlines = 0;
        for i in (0..n).rev() {
            indices[i] += 1;
            if indices[i] < dimensions[i] {
                break;
            }
            indices[i] = 0;
            newlines += 1;
        }
        if newlines == 0 && n > 0 {
            b.push(',');
        }
    }
    newline(b, indent);
    b.push(']');
}

fn unparse_dictionary(d: &Dictionary, b: &mut String, indent: usize) {
    b.push('{');
    for (i, (key, value)) in d.iter().enumerate() {
        if i > 0 {
            b.push(',');
        }
        newline(b, indent + 1);
        if needs_quotes(key) {
            b.push('"');
            escape(key, b);
            b.push('"');
        } else {
            b.push_str(key);
        }
        b.push_str(": ");
        unparse(value, b, indent + 1);
    }
    newline(b, indent);
    b.push('}');
}

/// Whether key must be quoted to be read back unchanged.
fn needs_quotes(key: &str) -> bool {
    let (Some(first), Some(last)) = (key.chars().next(), key.chars().last()) else {
        return true;
    };
    is_java_whitespace(first)
        || is_java_whitespace(last)
        || RESERVED_START.contains(first)
        || key
            .chars()
            .any(|c| RESERVED_ANYWHERE.contains(c) || c.is_control())
}

/// Whether c is whitespace as defined by Java's Character.isWhitespace.
fn is_java_whitespace(c: char) -> bool {
    matches!(c, '\t'..='\r' | '\u{1c}'..='\u{1f}')
        || c.is_whitespace() && !matches!(c, '\u{85}' | '\u{a0}' | '\u{2007}' | '\u{202f}')
}
