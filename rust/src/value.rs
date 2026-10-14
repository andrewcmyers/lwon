use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use crate::{Error, Parser};

/// A position in LWON input: a line number, starting at 1, and a column,
/// starting at 1 for the first character on a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

/// An LWON value. Values produced by a [`Parser`] record the location where
/// they begin; equality of values ignores locations.
///
/// The [`Display`](fmt::Display) form of a value is its LWON text, with all
/// strings quoted and keys quoted where needed to read them back unchanged.
/// Parsing the text gives back an equal value, except for arrays whose shape
/// LWON text cannot express: an array with no dimensions is read back as one
/// with dimensions `[1]`; leading dimensions of size 1 are lost, so that
/// dimensions `[1, 3]` are read back as `[3]`; and an empty array of more than
/// one dimension is read back as one with dimensions `[0]`.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(Text),
    Array(Array),
    Dictionary(Dictionary),
}

impl Value {
    /// The location where the value begins in the input, if it was parsed.
    pub fn location(&self) -> Option<Location> {
        match self {
            Value::Text(t) => t.location(),
            Value::Array(a) => a.location(),
            Value::Dictionary(d) => d.location(),
        }
    }

    /// The string, if this value is text.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Text(t) => Some(t),
            _ => None,
        }
    }

    /// The array, if this value is an array.
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// The dictionary, if this value is a dictionary.
    pub fn as_dictionary(&self) -> Option<&Dictionary> {
        match self {
            Value::Dictionary(d) => Some(d),
            _ => None,
        }
    }
}

impl From<Text> for Value {
    fn from(t: Text) -> Value {
        Value::Text(t)
    }
}

impl From<Array> for Value {
    fn from(a: Array) -> Value {
        Value::Array(a)
    }
}

impl From<Dictionary> for Value {
    fn from(d: Dictionary) -> Value {
        Value::Dictionary(d)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::Text(s.into())
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::Text(s.into())
    }
}

/// Parses the single LWON value in a string, which is named `"<string>"` in
/// errors. Fails if the string does not contain exactly one value.
impl FromStr for Value {
    type Err = Error;

    fn from_str(text: &str) -> Result<Value, Error> {
        Parser::new(text, "<string>").parse_only()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = String::new();
        crate::unparse::unparse(self, &mut b, 0);
        f.write_str(&b)
    }
}

/// LWON text: a string. It dereferences to [`str`].
#[derive(Clone, Debug, Eq)]
pub struct Text {
    value: String,
    location: Option<Location>,
}

impl Text {
    /// Text containing value, with no location.
    pub fn new(value: impl Into<String>) -> Text {
        Text {
            value: value.into(),
            location: None,
        }
    }

    pub(crate) fn at(value: String, location: Location) -> Text {
        Text {
            value,
            location: Some(location),
        }
    }

    /// The string.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// The string, consuming the text.
    pub fn into_string(self) -> String {
        self.value
    }

    /// The location where the text begins in the input, if it was parsed.
    pub fn location(&self) -> Option<Location> {
        self.location
    }
}

impl Deref for Text {
    type Target = str;

    fn deref(&self) -> &str {
        &self.value
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Text) -> bool {
        self.value == other.value
    }
}

impl From<String> for Text {
    fn from(s: String) -> Text {
        Text::new(s)
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Text {
        Text::new(s)
    }
}

/// A multidimensional LWON array. Its elements are stored in row-major order:
/// the last index varies fastest. For example, the rows `a, b` and `c, d`
/// form an array with dimensions `[2, 2]` and elements `a, b, c, d`.
///
/// An array with no dimensions holds a single element.
#[derive(Clone, Debug)]
pub struct Array {
    dimensions: Vec<usize>,
    elements: Vec<Value>,
    location: Option<Location>,
}

impl Array {
    /// The array with the given dimensions and elements in row-major order,
    /// with no location.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements is not the product of the dimensions.
    pub fn new(dimensions: Vec<usize>, elements: Vec<Value>) -> Array {
        assert_eq!(
            elements.len(),
            dimensions.iter().product::<usize>(),
            "the number of elements must be the product of the dimensions"
        );
        Array {
            dimensions,
            elements,
            location: None,
        }
    }

    pub(crate) fn at(
        dimensions: Vec<usize>,
        elements: Vec<Value>,
        location: Option<Location>,
    ) -> Array {
        Array {
            dimensions,
            elements,
            location,
        }
    }

    /// The size of each dimension, outermost first.
    pub fn dimensions(&self) -> &[usize] {
        &self.dimensions
    }

    /// The elements, in row-major order.
    pub fn elements(&self) -> &[Value] {
        &self.elements
    }

    /// The elements in row-major order, consuming the array.
    pub fn into_elements(self) -> Vec<Value> {
        self.elements
    }

    /// The number of elements.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Whether there are no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// The elements, in row-major order.
    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.elements.iter()
    }

    /// The element at the given indices, one per dimension; `None` if the
    /// number of indices is not the number of dimensions or an index is out
    /// of range.
    pub fn get(&self, indices: &[usize]) -> Option<&Value> {
        if indices.len() != self.dimensions.len() {
            return None;
        }
        let mut i = 0;
        for (&index, &size) in indices.iter().zip(&self.dimensions) {
            if index >= size {
                return None;
            }
            i = i * size + index;
        }
        self.elements.get(i)
    }

    /// A copy of this array without its dimensions of size 1, so that, for
    /// example, a single column becomes a one-dimensional array. A copy of an
    /// empty array is one-dimensional.
    pub fn squeeze(&self) -> Array {
        let dimensions = if self.is_empty() {
            vec![0]
        } else {
            self.dimensions.iter().copied().filter(|&n| n > 1).collect()
        };
        Array {
            dimensions,
            elements: self.elements.clone(),
            location: self.location,
        }
    }

    /// The location where the array begins in the input, if it was parsed.
    pub fn location(&self) -> Option<Location> {
        self.location
    }
}

/// Equal arrays have the same dimensions and equal elements.
impl PartialEq for Array {
    fn eq(&self, other: &Array) -> bool {
        self.dimensions == other.dimensions && self.elements == other.elements
    }
}

/// A one-dimensional array.
impl From<Vec<Value>> for Array {
    fn from(elements: Vec<Value>) -> Array {
        Array {
            dimensions: vec![elements.len()],
            elements,
            location: None,
        }
    }
}

/// A one-dimensional array.
impl FromIterator<Value> for Array {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Array {
        Array::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<'a> IntoIterator for &'a Array {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

/// An LWON dictionary: a sequence of entries (key-value pairs) in which keys
/// may be repeated.
#[derive(Clone, Debug, Default)]
pub struct Dictionary {
    entries: Vec<(String, Value)>,
    location: Option<Location>,
}

impl Dictionary {
    /// An empty dictionary with no location.
    pub fn new() -> Dictionary {
        Dictionary::default()
    }

    pub(crate) fn at(location: Location) -> Dictionary {
        Dictionary {
            entries: Vec::new(),
            location: Some(location),
        }
    }

    /// Add the entry `key: value` after all existing entries.
    pub fn push(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.entries.push((key.into(), value.into()));
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries, in order.
    pub fn entries(&self) -> &[(String, Value)] {
        &self.entries
    }

    /// The entries in order, consuming the dictionary.
    pub fn into_entries(self) -> Vec<(String, Value)> {
        self.entries
    }

    /// The entries, in order.
    pub fn iter(&self) -> std::slice::Iter<'_, (String, Value)> {
        self.entries.iter()
    }

    /// The values of the entries with the given key, in order.
    pub fn get_all<'a, 'k>(
        &'a self,
        key: &'k str,
    ) -> impl Iterator<Item = &'a Value> + use<'a, 'k> {
        self.entries
            .iter()
            .filter(move |(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// The value of the first entry with the given key, if any.
    pub fn get_first(&self, key: &str) -> Option<&Value> {
        self.get_all(key).next()
    }

    /// The value of the last entry with the given key, if any.
    pub fn get_last(&self, key: &str) -> Option<&Value> {
        self.get_all(key).last()
    }

    /// Whether some entry has the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.get_first(key).is_some()
    }

    /// The location where the dictionary begins in the input, if it was parsed.
    pub fn location(&self) -> Option<Location> {
        self.location
    }
}

/// Equal dictionaries have equal entries in the same order.
impl PartialEq for Dictionary {
    fn eq(&self, other: &Dictionary) -> bool {
        self.entries == other.entries
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Dictionary {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Dictionary {
        let mut d = Dictionary::new();
        for (k, v) in iter {
            d.push(k, v);
        }
        d
    }
}

impl<'a> IntoIterator for &'a Dictionary {
    type Item = &'a (String, Value);
    type IntoIter = std::slice::Iter<'a, (String, Value)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}
