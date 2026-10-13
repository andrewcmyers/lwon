//! Reading and writing LWON (LightWeight Object Notation).
//!
//! LWON values are [`Value`]s: [`Text`], multidimensional [`Array`]s, and
//! [`Dictionary`]s, which are sequences of entries whose keys may repeat.
//!
//! ```
//! use lwon::{Parser, Value};
//!
//! let value: Value = "{ name: Alice Smith\n  friends [ bob, carol ] }".parse()?;
//! let person = value.as_dictionary().unwrap();
//! assert_eq!(person.get_first("name").and_then(Value::as_str), Some("Alice Smith"));
//!
//! // An implicit array, such as CSV
//! let mut parser = Parser::new("a, b\nc, d\n", "table.csv");
//! let table = parser.parse_implicit_array()?.unwrap();
//! assert_eq!(table.dimensions(), [2, 2]);
//! assert_eq!(table.get(&[1, 0]).and_then(Value::as_str), Some("c"));
//!
//! println!("{value}"); // LWON text
//! # Ok::<(), lwon::Error>(())
//! ```
//!
//! # Deserializing with serde
//!
//! With the `serde` feature, which is enabled by default, [`from_str`] and
//! [`from_value`] deserialize LWON values into any type that implements
//! [`serde::Deserialize`], and [`to_string`] and [`to_value`] serialize any
//! type that implements [`serde::Serialize`] (see
//! [below](#serializing-with-serde)):
//!
//! ```
//! # #[cfg(feature = "serde")] {
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Person {
//!     name: String,
//!     height: u32,
//!     #[serde(rename = "friend")]
//!     friends: Vec<String>,
//! }
//!
//! let p: Person = lwon::from_str("{ name: Alice\n height: 69\n friend: bob\n friend: carol }")?;
//! assert_eq!(p.height, 69);
//! assert_eq!(p.friends, ["bob", "carol"]);
//!
//! // An implicit dictionary, such as a configuration file
//! let mut parser = lwon::Parser::new("name: Bob\nheight: 70\nfriend: alice\n", "bob.lwon");
//! let value = lwon::Value::from(parser.parse_implicit_dictionary()?.unwrap());
//! let p: Person = lwon::from_value(&value)?;
//! assert_eq!(p.friends, ["alice"]);
//! # }
//! # Ok::<(), lwon::Error>(())
//! ```
//!
//! LWON values represent Rust data as follows:
//!
//! - Text represents strings and characters, and also numbers and booleans
//!   (`true` and `false`), which are parsed from it. Empty text represents
//!   `()`, and text naming a unit variant represents that enum variant.
//! - An array represents a sequence, tuple, or struct, whose fields are its
//!   elements in order. An array of more than one dimension is a sequence of
//!   arrays, one for each index of its first dimension.
//! - A dictionary represents a map or struct. A dictionary with a single
//!   entry also represents an enum variant: the key is the variant's name
//!   and the value is its data.
//! - A repeated key's values, in order, form a sequence, which is an error
//!   where a sequence is not expected. Where a sequence is expected for a key
//!   that is not repeated, the key's value is a sequence of one value unless
//!   it is an array.
//! - A value present for an `Option` represents `Some`. A missing struct field
//!   of type `Option` is `None`.
//!
//! Errors from deserialization give the location of the value that caused
//! them.
//!
//! [`Value`] itself implements `Deserialize` and `Serialize`, so part of the
//! data can be left untyped by giving a field the type `Value`. A `Value`
//! deserialized from LWON is an exact copy, including its locations, except
//! that the values of a repeated key form a one-dimensional array. `Value` can
//! also be converted to and from other serde formats, such as JSON; numbers
//! and booleans from them become text, and null becomes empty text.
//!
//! # Serializing with serde
//!
//! Serialization represents Rust data as deserialization does, so that
//! deserializing the result gives back the original data:
//!
//! ```
//! # #[cfg(feature = "serde")] {
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Table {
//!     title: Option<String>,
//!     rows: Vec<(String, u32)>,
//! }
//!
//! let table = Table { title: None, rows: vec![("a".into(), 1), ("b".into(), 2)] };
//! let text = lwon::to_string(&table)?;
//! assert_eq!(text, "{\n  rows: [\n    \"a\", \"1\"\n    \"b\", \"2\"\n  ]\n}");
//! assert_eq!(lwon::from_str::<Table>(&text)?, table);
//! # }
//! # Ok::<(), lwon::Error>(())
//! ```
//!
//! In addition:
//!
//! - Text is written as a quoted string.
//! - A struct field or map entry whose value is `None` is omitted. `None`
//!   cannot be serialized elsewhere.
//! - Bytes are written as text, and must be UTF-8.
//! - Two or more sequences or tuples of the same nonempty shape are combined
//!   into an array of one more dimension, so that, for example, a
//!   `Vec<Vec<T>>` is written as rows. Otherwise, a sequence is a
//!   one-dimensional array.
//! - Map keys are written as text, and must be strings, characters,
//!   numbers, booleans, or unit variants.

#[cfg(feature = "serde")]
mod de;
mod error;
mod parser;
mod scanner;
#[cfg(feature = "serde")]
mod ser;
mod unparse;
mod value;

#[cfg(feature = "serde")]
pub use de::{from_str, from_value};
pub use error::Error;
pub use parser::Parser;
#[cfg(feature = "serde")]
pub use ser::{to_string, to_value};
pub use value::{Array, Dictionary, Location, Text, Value};
