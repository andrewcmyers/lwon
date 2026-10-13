use serde::Serialize;
use serde::ser::{
    self, Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

use crate::{Array, Dictionary, Error, Value};

/// The LWON value representing value, as described in the
/// [crate documentation](crate#serializing-with-serde). Fails if value
/// is `None` or contains a `None` that is not a struct field or map value, if
/// it contains bytes that are not UTF-8, or if it contains a map key that is
/// not a string, character, number, boolean, or unit variant.
pub fn to_value<T: Serialize + ?Sized>(value: &T) -> Result<Value, Error> {
    present(value.serialize(ValueSerializer)?)
}

/// The LWON text for value, which is as for [`to_value`].
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String, Error> {
    Ok(to_value(value)?.to_string())
}

/// The value v, or an error if it is absent (`None`).
fn present(v: Option<Value>) -> Result<Value, Error> {
    v.ok_or_else(|| {
        ser::Error::custom("None can be represented only by omitting a struct field or map entry")
    })
}

fn text(s: impl Into<String>) -> Result<Option<Value>, Error> {
    Ok(Some(Value::from(s.into())))
}

/// The dictionary with the single entry key: value.
fn single_entry(key: &str, value: Value) -> Option<Value> {
    let mut d = Dictionary::new();
    d.push(key, value);
    Some(d.into())
}

/// The array with the given elements. If there are at least two elements and
/// they are nonempty arrays of the same dimensions, they are combined into an
/// array of one more dimension; otherwise, the array is one-dimensional.
pub(crate) fn array_of(elements: Vec<Value>) -> Array {
    if elements.len() >= 2
        && let Some(Value::Array(first)) = elements.first()
        && !first.is_empty()
    {
        let mut dimensions = vec![elements.len()];
        dimensions.extend_from_slice(first.dimensions());
        if elements
            .iter()
            .all(|e| matches!(e, Value::Array(a) if a.dimensions() == &dimensions[1..]))
        {
            let elements = elements
                .into_iter()
                .flat_map(|e| match e {
                    Value::Array(a) => a.into_elements(),
                    _ => unreachable!("all elements are arrays"),
                })
                .collect();
            return Array::new(dimensions, elements);
        }
    }
    Array::from(elements)
}

/// A value is serialized as a string if it is text, as a map with its entries
/// in order (including repeated keys) if it is a dictionary, and as a sequence
/// if it is an array. An array of more than one dimension is a sequence of
/// arrays, one for each index of its first dimension.
impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Text(t) => serializer.serialize_str(t),
            Value::Array(a) => ArraySlice {
                dimensions: a.dimensions(),
                elements: a.elements(),
            }
            .serialize(serializer),
            Value::Dictionary(d) => {
                let mut map = serializer.serialize_map(Some(d.len()))?;
                for (key, value) in d {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

/// An array, or the subarray at one index of an array's outer dimensions.
struct ArraySlice<'a> {
    dimensions: &'a [usize],
    elements: &'a [Value],
}

impl Serialize for ArraySlice<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.dimensions.len() <= 1 {
            return serializer.collect_seq(self.elements);
        }
        let size = self.elements.len() / self.dimensions[0];
        serializer.collect_seq((0..self.dimensions[0]).map(|i| ArraySlice {
            dimensions: &self.dimensions[1..],
            elements: &self.elements[i * size..(i + 1) * size],
        }))
    }
}

/// Serializes data as an LWON value, or as `None` if the data is `None`.
struct ValueSerializer;

/// Defines Serializer methods that write their argument as text.
macro_rules! as_text {
    ($($method:ident($ty:ty);)*) => {$(
        fn $method(self, v: $ty) -> Result<Option<Value>, Error> {
            text(v.to_string())
        }
    )*};
}

impl Serializer for ValueSerializer {
    type Ok = Option<Value>;
    type Error = Error;
    type SerializeSeq = SeqBuilder;
    type SerializeTuple = SeqBuilder;
    type SerializeTupleStruct = SeqBuilder;
    type SerializeTupleVariant = VariantBuilder<SeqBuilder>;
    type SerializeMap = DictionaryBuilder;
    type SerializeStruct = DictionaryBuilder;
    type SerializeStructVariant = VariantBuilder<DictionaryBuilder>;

    as_text! {
        serialize_bool(bool); serialize_i8(i8); serialize_i16(i16); serialize_i32(i32);
        serialize_i64(i64); serialize_i128(i128); serialize_u8(u8); serialize_u16(u16);
        serialize_u32(u32); serialize_u64(u64); serialize_u128(u128); serialize_f32(f32);
        serialize_f64(f64); serialize_char(char); serialize_str(&str);
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Option<Value>, Error> {
        match std::str::from_utf8(v) {
            Ok(s) => text(s),
            Err(_) => Err(ser::Error::custom("bytes must be UTF-8")),
        }
    }

    fn serialize_none(self) -> Result<Option<Value>, Error> {
        Ok(None)
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Option<Value>, Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Option<Value>, Error> {
        text("")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Option<Value>, Error> {
        text("")
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<Option<Value>, Error> {
        text(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Option<Value>, Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Option<Value>, Error> {
        Ok(single_entry(variant, to_value(value)?))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqBuilder, Error> {
        Ok(SeqBuilder {
            elements: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqBuilder, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SeqBuilder, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<VariantBuilder<SeqBuilder>, Error> {
        Ok(VariantBuilder {
            variant,
            data: self.serialize_seq(Some(len))?,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<DictionaryBuilder, Error> {
        Ok(DictionaryBuilder {
            dictionary: Dictionary::new(),
            key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<DictionaryBuilder, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<VariantBuilder<DictionaryBuilder>, Error> {
        Ok(VariantBuilder {
            variant,
            data: self.serialize_map(Some(len))?,
        })
    }
}

/// Builds an array from serialized elements.
struct SeqBuilder {
    elements: Vec<Value>,
}

impl SerializeSeq for SeqBuilder {
    type Ok = Option<Value>;
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Option<Value>, Error> {
        Ok(Some(array_of(self.elements).into()))
    }
}

impl SerializeTuple for SeqBuilder {
    type Ok = Option<Value>;
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Option<Value>, Error> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for SeqBuilder {
    type Ok = Option<Value>;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Option<Value>, Error> {
        SerializeSeq::end(self)
    }
}

/// Builds a dictionary from serialized entries, omitting those whose values
/// are `None`.
struct DictionaryBuilder {
    dictionary: Dictionary,
    key: Option<String>,
}

impl SerializeMap for DictionaryBuilder {
    type Ok = Option<Value>;
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        self.key = Some(key.serialize(KeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        let key = self
            .key
            .take()
            .expect("serialize_value before serialize_key");
        if let Some(value) = value.serialize(ValueSerializer)? {
            self.dictionary.push(key, value);
        }
        Ok(())
    }

    fn end(self) -> Result<Option<Value>, Error> {
        Ok(Some(self.dictionary.into()))
    }
}

impl SerializeStruct for DictionaryBuilder {
    type Ok = Option<Value>;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        if let Some(value) = value.serialize(ValueSerializer)? {
            self.dictionary.push(key, value);
        }
        Ok(())
    }

    fn end(self) -> Result<Option<Value>, Error> {
        SerializeMap::end(self)
    }
}

/// Builds an enum variant with data, as a dictionary with a single entry whose
/// key is the variant's name and whose value is the data.
struct VariantBuilder<B> {
    variant: &'static str,
    data: B,
}

impl SerializeTupleVariant for VariantBuilder<SeqBuilder> {
    type Ok = Option<Value>;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        SerializeSeq::serialize_element(&mut self.data, value)
    }

    fn end(self) -> Result<Option<Value>, Error> {
        Ok(single_entry(
            self.variant,
            present(SerializeSeq::end(self.data)?)?,
        ))
    }
}

impl SerializeStructVariant for VariantBuilder<DictionaryBuilder> {
    type Ok = Option<Value>;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        SerializeStruct::serialize_field(&mut self.data, key, value)
    }

    fn end(self) -> Result<Option<Value>, Error> {
        Ok(single_entry(
            self.variant,
            present(SerializeMap::end(self.data)?)?,
        ))
    }
}

/// Serializes a map key as a string.
struct KeySerializer;

fn key_error() -> Error {
    ser::Error::custom(
        "dictionary keys must be strings, characters, numbers, booleans, or unit variants",
    )
}

/// Defines Serializer methods that write their argument as a string.
macro_rules! key_as_string {
    ($($method:ident($ty:ty);)*) => {$(
        fn $method(self, v: $ty) -> Result<String, Error> {
            Ok(v.to_string())
        }
    )*};
}

impl Serializer for KeySerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    key_as_string! {
        serialize_bool(bool); serialize_i8(i8); serialize_i16(i16); serialize_i32(i32);
        serialize_i64(i64); serialize_i128(i128); serialize_u8(u8); serialize_u16(u16);
        serialize_u32(u32); serialize_u64(u64); serialize_u128(u128); serialize_f32(f32);
        serialize_f64(f64); serialize_char(char); serialize_str(&str);
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<String, Error> {
        Err(key_error())
    }

    fn serialize_none(self) -> Result<String, Error> {
        Err(key_error())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, _value: &T) -> Result<String, Error> {
        Err(key_error())
    }

    fn serialize_unit(self) -> Result<String, Error> {
        Err(key_error())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, Error> {
        Err(key_error())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<String, Error> {
        Ok(variant.to_string())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<String, Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, Error> {
        Err(key_error())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(key_error())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(key_error())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(key_error())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(key_error())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(key_error())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Err(key_error())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(key_error())
    }
}
