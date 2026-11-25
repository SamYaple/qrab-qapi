//! A Serde serializer that writes down the calls of Serde's data model that
//! JSON does not show: the names, indices and lengths a value passes.
#![allow(dead_code, reason = "each test binary uses a part of the recorder")]

use serde::ser::{self, Impossible, Serialize};

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}

/// The calls that serializing `value` makes, such as
/// `unit_variant(Name, 0, wire)` or `struct(Name, 2) a b`.
pub fn record<T: Serialize + ?Sized>(value: &T) -> Result<String, Error> {
    value.serialize(Recorder)
}

pub struct Recorder;

pub struct Struct(String);

impl ser::SerializeStruct for Struct {
    type Ok = String;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        _value: &T,
    ) -> Result<(), Error> {
        self.0.push(' ');
        self.0.push_str(key);
        Ok(())
    }

    fn end(self) -> Result<String, Error> {
        Ok(self.0)
    }
}

macro_rules! scalar {
    ($($method:ident: $ty:ty),*) => {
        $(fn $method(self, value: $ty) -> Result<String, Error> {
            Ok(format!("{}({value:?})", stringify!($method)))
        })*
    };
}

impl ser::Serializer for Recorder {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Struct;
    type SerializeStructVariant = Impossible<String, Error>;

    scalar!(
        serialize_bool: bool, serialize_i8: i8, serialize_i16: i16, serialize_i32: i32,
        serialize_i64: i64, serialize_u8: u8, serialize_u16: u16, serialize_u32: u32,
        serialize_u64: u64, serialize_f32: f32, serialize_f64: f64, serialize_char: char,
        serialize_str: &str, serialize_bytes: &[u8]
    );

    fn serialize_none(self) -> Result<String, Error> {
        Ok("none".into())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<String, Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<String, Error> {
        Ok("unit".into())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<String, Error> {
        Ok(format!("unit_struct({name})"))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
    ) -> Result<String, Error> {
        Ok(format!("unit_variant({name}, {index}, {variant})"))
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<String, Error> {
        Ok(format!("newtype_struct({name})"))
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<String, Error> {
        Ok(format!("newtype_variant({name}, {index}, {variant})"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(Error("seq".into()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(Error("tuple".into()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(Error("tuple_struct".into()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(Error("tuple_variant".into()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(Error("map".into()))
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Struct, Error> {
        Ok(Struct(format!("struct({name}, {len})")))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(Error("struct_variant".into()))
    }
}
