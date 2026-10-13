use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;

use serde::de::value::BorrowedStrDeserializer;
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, Expected, MapAccess, SeqAccess,
    Unexpected, VariantAccess, Visitor,
};
use serde::{Deserialize, Deserializer, forward_to_deserialize_any};

use crate::ser::array_of;
use crate::{Array, Dictionary, Error, Location, Value};

/// Deserialize an instance of `T` from the single LWON value in text, as
/// described in the [crate documentation](crate#deserializing-with-serde).
/// Errors are named `"<string>"`. Fails if text does not contain exactly one
/// value or the value does not represent a `T`.
pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, Error> {
    let value: Value = text.parse()?;
    from_value(&value).map_err(|e| e.named("<string>"))
}

/// Deserialize an instance of `T` from value, as described in the
/// [crate documentation](crate#deserializing-with-serde). Errors give the
/// location of the offending value if it was parsed, but no name. Fails if
/// value does not represent a `T`.
pub fn from_value<'de, T: Deserialize<'de>>(value: &'de Value) -> Result<T, Error> {
    T::deserialize(value)
}

/// The name of the newtype struct that `Value::deserialize` requests. The
/// deserializers in this crate respond with an exact copy of the value, keeping
/// its repeated keys, array dimensions, and locations; other deserializers treat
/// the newtype struct as its contents.
const VALUE_TOKEN: &str = "$lwon::private::Value";

thread_local! {
    /// The copy of a value being passed to ValueVisitor by visit_copy.
    static COPY: Cell<Option<Value>> = const { Cell::new(None) };
}

/// Pass value to visitor, which is a ValueVisitor. A Visitor can receive only
/// primitive data, so the value travels through COPY, and the visitor
/// receives VALUE_TOKEN as text telling it to take the value from there.
fn visit_copy<'de, V: Visitor<'de>>(value: Value, visitor: V) -> Result<V::Value, Error> {
    COPY.set(Some(value));
    let result = visitor.visit_borrowed_str(VALUE_TOKEN);
    COPY.take();
    result
}

/// A value is deserialized from strings, which become text; from booleans,
/// numbers, and characters, which become text representing them; from
/// sequences, which become arrays as in [`to_value`](crate::to_value); and
/// from maps, whose entries become dictionary entries. A unit or null value
/// becomes empty text. A value deserialized from LWON by [`from_value`] or
/// [`from_str`] is an exact copy of the LWON value, including its locations,
/// except that the values of a repeated key form a one-dimensional array.
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_newtype_struct(VALUE_TOKEN, ValueVisitor)
    }
}

struct ValueVisitor;

/// Defines Visitor methods that represent their argument as text.
macro_rules! visit_as_text {
    ($($method:ident($ty:ty);)*) => {$(
        fn $method<E: de::Error>(self, v: $ty) -> Result<Value, E> {
            Ok(Value::from(v.to_string()))
        }
    )*};
}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an LWON value")
    }

    visit_as_text! {
        visit_bool(bool); visit_i64(i64); visit_i128(i128); visit_u64(u64);
        visit_u128(u128); visit_f64(f64); visit_char(char);
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        if v == VALUE_TOKEN
            && let Some(value) = COPY.take()
        {
            return Ok(value);
        }
        Ok(Value::from(v))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::from(v))
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Value, E> {
        match std::str::from_utf8(v) {
            Ok(s) => Ok(Value::from(s)),
            Err(_) => Err(E::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::from(""))
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        Value::deserialize(deserializer)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::from(""))
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut elements = Vec::new();
        while let Some(element) = seq.next_element()? {
            elements.push(element);
        }
        Ok(array_of(elements).into())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut d = Dictionary::new();
        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            d.push(key, value);
        }
        Ok(d.into())
    }
}

fn unexpected(value: &Value) -> Unexpected<'_> {
    match value {
        Value::Text(t) => Unexpected::Str(t),
        Value::Array(_) => Unexpected::Seq,
        Value::Dictionary(_) => Unexpected::Map,
    }
}

/// Defines Deserializer methods that call the methods of the same names on
/// `self.$inner()` and give errors the location of `self`.
macro_rules! located {
    ($inner:ident; $($method:ident($($arg:ident: $ty:ty),*);)*) => {$(
        fn $method<V: Visitor<'de>>(self, $($arg: $ty,)* visitor: V) -> Result<V::Value, Error> {
            let location = self.location();
            self.$inner().$method($($arg,)* visitor).map_err(|e| e.at(location))
        }
    )*};
}

/// Lists every Deserializer method except deserialize_any, option,
/// newtype_struct, seq, and ignored_any, passing them to the macro $m.
macro_rules! other_methods {
    ($m:ident; $($prefix:tt)*) => {
        $m! { $($prefix)*
            deserialize_bool(); deserialize_i8(); deserialize_i16(); deserialize_i32();
            deserialize_i64(); deserialize_i128(); deserialize_u8(); deserialize_u16();
            deserialize_u32(); deserialize_u64(); deserialize_u128(); deserialize_f32();
            deserialize_f64(); deserialize_char(); deserialize_str(); deserialize_string();
            deserialize_bytes(); deserialize_byte_buf(); deserialize_unit();
            deserialize_unit_struct(name: &'static str);
            deserialize_tuple(len: usize);
            deserialize_tuple_struct(name: &'static str, len: usize);
            deserialize_map();
            deserialize_struct(name: &'static str, fields: &'static [&'static str]);
            deserialize_enum(name: &'static str, variants: &'static [&'static str]);
            deserialize_identifier();
        }
    };
}

impl<'de> Deserializer<'de> for &'de Value {
    type Error = Error;

    other_methods!(located; inner;);
    located! { inner;
        deserialize_any(); deserialize_seq();
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        if name == VALUE_TOKEN {
            return visit_copy(self.clone(), visitor);
        }
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
}

impl Value {
    fn inner(&self) -> ValueDeserializer<'_> {
        ValueDeserializer(self)
    }
}

/// Deserializes a value, without attaching its location to errors.
struct ValueDeserializer<'de>(&'de Value);

impl<'de> ValueDeserializer<'de> {
    /// The text of the value, or an error if it is not text.
    fn text(&self, expected: &dyn Expected) -> Result<&'de str, Error> {
        match self.0 {
            Value::Text(t) => Ok(t.as_str()),
            v => Err(de::Error::invalid_type(unexpected(v), expected)),
        }
    }

    fn invalid_type<T>(&self, expected: &dyn Expected) -> Result<T, Error> {
        Err(de::Error::invalid_type(unexpected(self.0), expected))
    }
}

/// Defines Deserializer methods for numbers, which are parsed from text.
macro_rules! numbers {
    ($($method:ident => $visit:ident,)*) => {$(
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let s = self.text(&visitor)?;
            match s.parse() {
                Ok(n) => visitor.$visit(n),
                Err(_) => Err(de::Error::invalid_value(Unexpected::Str(s), &visitor)),
            }
        }
    )*};
}

/// Defines the Deserializer methods for types represented by text, using
/// `self.text()` to obtain the text.
macro_rules! text_methods {
    () => {
        fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            match self.text(&visitor)? {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                s => Err(de::Error::invalid_value(Unexpected::Str(s), &visitor)),
            }
        }

        numbers! {
            deserialize_i8 => visit_i8, deserialize_i16 => visit_i16,
            deserialize_i32 => visit_i32, deserialize_i64 => visit_i64,
            deserialize_i128 => visit_i128, deserialize_u8 => visit_u8,
            deserialize_u16 => visit_u16, deserialize_u32 => visit_u32,
            deserialize_u64 => visit_u64, deserialize_u128 => visit_u128,
            deserialize_f32 => visit_f32, deserialize_f64 => visit_f64,
        }

        fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let s = self.text(&visitor)?;
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => visitor.visit_char(c),
                _ => Err(de::Error::invalid_value(Unexpected::Str(s), &visitor)),
            }
        }

        fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let s = self.text(&visitor)?;
            visitor.visit_borrowed_str(s)
        }

        fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            self.deserialize_str(visitor)
        }

        fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            self.deserialize_str(visitor)
        }

        fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let s = self.text(&visitor)?;
            visitor.visit_borrowed_bytes(s.as_bytes())
        }

        fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            self.deserialize_bytes(visitor)
        }

        fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            match self.text(&visitor)? {
                "" => visitor.visit_unit(),
                s => Err(de::Error::invalid_value(Unexpected::Str(s), &visitor)),
            }
        }

        fn deserialize_unit_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error> {
            self.deserialize_unit(visitor)
        }
    };
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Text(t) => visitor.visit_borrowed_str(t.as_str()),
            Value::Array(a) => ArrayView::of(a).deserialize_any(visitor),
            Value::Dictionary(d) => visitor.visit_map(DictionaryAccess::new(d)),
        }
    }

    text_methods!();

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_some(self.0)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self.0)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Array(a) => ArrayView::of(a).deserialize_any(visitor),
            _ => self.invalid_type(&visitor),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Dictionary(d) => visitor.visit_map(DictionaryAccess::new(d)),
            _ => self.invalid_type(&visitor),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.0 {
            Value::Dictionary(d) => visitor.visit_map(DictionaryAccess::new(d)),
            Value::Array(a) => ArrayView::of(a).deserialize_any(visitor),
            _ => self.invalid_type(&visitor),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.0 {
            Value::Text(t) => visitor.visit_enum(BorrowedStrDeserializer::new(t.as_str())),
            Value::Dictionary(d) if d.len() == 1 => {
                let (key, value) = &d.entries()[0];
                visitor.visit_enum(Variant { key, value })
            }
            _ => self.invalid_type(&visitor),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
}

/// Deserializes a dictionary key, which is text.
struct KeyDeserializer<'de>(&'de str);

impl<'de> KeyDeserializer<'de> {
    fn text(&self, _expected: &dyn Expected) -> Result<&'de str, Error> {
        Ok(self.0)
    }
}

impl<'de> Deserializer<'de> for KeyDeserializer<'de> {
    type Error = Error;

    text_methods!();

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_borrowed_str(self.0)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_enum(BorrowedStrDeserializer::new(self.0))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! { seq tuple tuple_struct map struct }
}

/// An array, or the subarray at one index of an array's outer dimensions.
#[derive(Clone, Copy)]
struct ArrayView<'de> {
    dimensions: &'de [usize],
    elements: &'de [Value],
    location: Option<Location>,
}

impl<'de> ArrayView<'de> {
    fn of(a: &'de Array) -> ArrayView<'de> {
        ArrayView {
            dimensions: a.dimensions(),
            elements: a.elements(),
            location: a.location(),
        }
    }

    /// The number of items in the outermost dimension.
    fn len(&self) -> usize {
        match self.dimensions.first() {
            Some(&n) if self.dimensions.len() > 1 => n,
            _ => self.elements.len(),
        }
    }
}

impl<'de> Deserializer<'de> for ArrayView<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visit_seq(
            visitor,
            ArrayAccess {
                array: self,
                next: 0,
            },
        )
        .map_err(|e| e.at(self.location))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        if name == VALUE_TOKEN {
            let array = Array::at(
                self.dimensions.to_vec(),
                self.elements.to_vec(),
                self.location,
            );
            return visit_copy(array.into(), visitor);
        }
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct enum identifier
    }
}

/// A sequence whose items can be counted.
trait Remaining<'de>: SeqAccess<'de, Error = Error> {
    /// The number of items already visited.
    fn visited(&self) -> usize;
    /// The number of items not yet visited.
    fn remaining(&self) -> usize;
}

/// Visit seq with visitor, failing if the visitor does not visit every item.
fn visit_seq<'de, V: Visitor<'de>>(
    visitor: V,
    mut seq: impl Remaining<'de>,
) -> Result<V::Value, Error> {
    let value = visitor.visit_seq(&mut seq)?;
    match seq.remaining() {
        0 => Ok(value),
        n => Err(de::Error::invalid_length(
            seq.visited() + n,
            &ExpectedLength(seq.visited()),
        )),
    }
}

struct ExpectedLength(usize);

impl Expected for ExpectedLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} elements", self.0)
    }
}

/// The items of an array: its elements if it has at most one dimension, and
/// otherwise its subarrays at each index of its outermost dimension.
struct ArrayAccess<'de> {
    array: ArrayView<'de>,
    next: usize,
}

impl<'de> SeqAccess<'de> for ArrayAccess<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        if self.next == self.array.len() {
            return Ok(None);
        }
        let i = self.next;
        self.next += 1;
        let dimensions = self.array.dimensions;
        if dimensions.len() <= 1 {
            return seed.deserialize(&self.array.elements[i]).map(Some);
        }
        let size = self.array.elements.len() / dimensions[0];
        let elements = &self.array.elements[i * size..(i + 1) * size];
        let location = elements.first().and_then(Value::location);
        seed.deserialize(ArrayView {
            dimensions: &dimensions[1..],
            elements,
            location,
        })
        .map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining())
    }
}

impl<'de> Remaining<'de> for ArrayAccess<'de> {
    fn visited(&self) -> usize {
        self.next
    }

    fn remaining(&self) -> usize {
        self.array.len() - self.next
    }
}

/// The values of a dictionary entry or of several entries with the same key.
struct ValuesAccess<'de> {
    values: Vec<&'de Value>,
    next: usize,
}

impl<'de> SeqAccess<'de> for ValuesAccess<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        let Some(&value) = self.values.get(self.next) else {
            return Ok(None);
        };
        self.next += 1;
        seed.deserialize(value).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining())
    }
}

impl<'de> Remaining<'de> for ValuesAccess<'de> {
    fn visited(&self) -> usize {
        self.next
    }

    fn remaining(&self) -> usize {
        self.values.len() - self.next
    }
}

/// The entries of a dictionary, with the values of each key grouped together
/// in the position of the key's first entry.
struct DictionaryAccess<'de> {
    entries: std::vec::IntoIter<Entry<'de>>,
    value: Option<Entry<'de>>,
}

impl<'de> DictionaryAccess<'de> {
    fn new(d: &'de Dictionary) -> DictionaryAccess<'de> {
        let mut entries: Vec<Entry<'de>> = Vec::new();
        let mut index: HashMap<&str, usize> = HashMap::new();
        for (key, value) in d {
            match index.get(key.as_str()) {
                Some(&i) => entries[i].values.push(value),
                None => {
                    index.insert(key, entries.len());
                    entries.push(Entry {
                        key,
                        values: vec![value],
                    });
                }
            }
        }
        DictionaryAccess {
            entries: entries.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for DictionaryAccess<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        let Some(entry) = self.entries.next() else {
            return Ok(None);
        };
        let location = entry.location();
        let key = seed.deserialize(KeyDeserializer(entry.key));
        self.value = Some(entry);
        key.map(Some).map_err(|e: Error| e.at(location))
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        seed.deserialize(
            self.value
                .take()
                .expect("next_value_seed before next_key_seed"),
        )
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.entries.len())
    }
}

/// The values of all entries of a dictionary with the same key. As a
/// sequence, they are the values, except that a single value that is not an
/// array is a sequence of one value. Otherwise, there must be a single value.
struct Entry<'de> {
    key: &'de str,
    values: Vec<&'de Value>,
}

impl<'de> Entry<'de> {
    fn location(&self) -> Option<Location> {
        self.values[0].location()
    }

    /// The single value, or an error if the key is repeated.
    fn single(self) -> Result<&'de Value, Error> {
        match self.values[..] {
            [value] => Ok(value),
            _ => Err(de::Error::custom(format!("key `{}` is repeated", self.key))),
        }
    }
}

/// Defines Deserializer methods that call the methods of the same names on the
/// single value of self.
macro_rules! single {
    ($($method:ident($($arg:ident: $ty:ty),*);)*) => {$(
        fn $method<V: Visitor<'de>>(self, $($arg: $ty,)* visitor: V) -> Result<V::Value, Error> {
            let location = self.values.get(1).and_then(|v| v.location());
            self.single().map_err(|e| e.at(location))?.$method($($arg,)* visitor)
        }
    )*};
}

impl<'de> Deserializer<'de> for Entry<'de> {
    type Error = Error;

    other_methods!(single;);

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.values[..] {
            [value] => value.deserialize_any(visitor),
            _ => visit_seq(
                visitor,
                ValuesAccess {
                    values: self.values,
                    next: 0,
                },
            ),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.values[..] {
            [value @ Value::Array(_)] => value.deserialize_seq(visitor),
            _ => visit_seq(
                visitor,
                ValuesAccess {
                    values: self.values,
                    next: 0,
                },
            ),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        if name != VALUE_TOKEN {
            return visitor.visit_newtype_struct(self);
        }
        match self.values[..] {
            [value] => value.deserialize_newtype_struct(name, visitor),
            _ => {
                let location = self.location();
                let values: Vec<Value> = self.values.into_iter().cloned().collect();
                visit_copy(
                    Array::at(vec![values.len()], values, location).into(),
                    visitor,
                )
            }
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
}

/// An enum variant written as a dictionary with a single entry, whose key is
/// the variant name and whose value is the variant's data.
struct Variant<'de> {
    key: &'de str,
    value: &'de Value,
}

impl<'de> EnumAccess<'de> for Variant<'de> {
    type Error = Error;
    type Variant = &'de Value;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, &'de Value), Error> {
        let variant = seed.deserialize(BorrowedStrDeserializer::new(self.key))?;
        Ok((variant, self.value))
    }
}

impl<'de> VariantAccess<'de> for &'de Value {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        <()>::deserialize(self)
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_struct("", fields, visitor)
    }
}
