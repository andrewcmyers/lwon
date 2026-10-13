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

mod error;
mod parser;
mod scanner;
mod unparse;
mod value;

pub use error::Error;
pub use parser::Parser;
pub use value::{Array, Dictionary, Location, Text, Value};
