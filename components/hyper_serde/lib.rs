// Copyright 2023 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate provides wrappers and convenience functions to make Hyper and
//! Serde work hand in hand.
//!
//! The supported types are:
//!
//! * `cookie::Cookie`
//! * `headers_ext::ContentType`
//! * `hyper::header::Headers`
//! * `hyper::StatusCode`
//! * `hyper::Method`
//! * `hyper::Uri`
//! * `mime::Mime`
//! * `time::Tm`
//!
//! # How do I use a data type with a `HeaderMap` member with Serde?
//!
//! Use the serde attributes `deserialize_with` and `serialize_with`.
//!
//! ```
//! struct MyStruct {
//! #[serde(deserialize_with = "hyper_serde::deserialize",
//! serialize_with = "hyper_serde::serialize")]
//! headers: HeaderMap,
//! }
//! ```
//!
//! # How do I encode a `HeaderMap` value with `serde_json::to_string`?
//!
//! Use the `Ser` wrapper.
//!
//! ```
//! serde_json::to_string(&Ser::new(&headers))
//! ```
//!
//! # How do I decode a `Method` value with `serde_json::parse`?
//!
//! Use the `De` wrapper.
//!
//! ```
//! serde_json::parse::<De<Method>>("\"PUT\"").map(De::into_inner)
//! ```
//!
//! # How do I send `Cookie` values as part of an IPC channel?
//!
//! Use the `Serde` wrapper. It implements `Deref` and `DerefMut` for
//! convenience.
//!
//! ```
//! ipc::channel::<Serde<Cookie>>()
//! ```
//!
//!

#![deny(missing_docs)]
#![deny(unsafe_code)]

use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::{cmp, fmt, str};

use cookie::Cookie;
use headers::ContentType;
use http::HeaderMap;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Method, StatusCode, Uri};
use mime::Mime;
use serde::de::{self, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::{ByteBuf, Bytes};
use time::{strptime, Tm};

/// Deserialises a `T` value with a given deserializer.
///
/// This is useful to deserialize Hyper types used in structure fields or
/// tuple members with `#[serde(deserialize_with = "hyper_serde::deserialize")]`.
#[inline(always)]
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    De<T>: Deserialize<'de>,
{
    De::deserialize(deserializer).map(De::into_inner)
}

/// Serialises `value` with a given serializer.
///
/// This is useful to serialize Hyper types used in structure fields or
/// tuple members with `#[serde(serialize_with = "hyper_serde::serialize")]`.
#[inline(always)]
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    for<'a> Ser<'a, T>: Serialize,
{
    Ser::new(value).serialize(serializer)
}

/// Serialises `value` with a given serializer in a pretty way.
///
/// This does the same job as `serialize` but with a prettier format
/// for some combinations of types and serialisers.
///
/// For now, the only change from `serialize` is when serialising `Headers`,
/// where the items in the header values get serialised as strings instead
/// of sequences of bytes, if they represent UTF-8 text.
#[inline(always)]
pub fn serialize_pretty<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    for<'a> Ser<'a, T>: Serialize,
{
    Ser::new_pretty(value).serialize(serializer)
}

/// A wrapper to deserialize Hyper types.
///
/// This is useful with functions such as `serde_json::from_str`.
///
/// Values of this type can only be obtained through
/// the `serde::Deserialize` trait.
#[derive(Debug, PartialEq)]
pub struct De<T> {
    v: T,
}

impl<T> De<T> {
    /// Returns a new `De` wrapper
    #[inline(always)]
    pub fn new(v: T) -> Self {
        De { v }
    }
}

impl<'de, T> De<T>
where
    De<T>: Deserialize<'de>,
{
    /// Consumes this wrapper, returning the deserialized value.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.v
    }
}

/// A wrapper to serialize Hyper types.
///
/// This is useful with functions such as `serde_json::to_string`.
///
/// Values of this type can only be passed to the `serde::Serialize` trait.
#[derive(Debug)]
pub struct Ser<'a, T: 'a> {
    v: &'a T,
    pretty: bool,
}

impl<'a, T> Ser<'a, T>
where
    Ser<'a, T>: serde::Serialize,
{
    /// Returns a new `Ser` wrapper.
    #[inline(always)]
    pub fn new(value: &'a T) -> Self {
        Ser {
            v: value,
            pretty: false,
        }
    }

    /// Returns a new `Ser` wrapper, in pretty mode.
    ///
    /// See `serialize_pretty`.
    #[inline(always)]
    pub fn new_pretty(value: &'a T) -> Self {
        Ser {
            v: value,
            pretty: true,
        }
    }
}

/// A convenience wrapper to be used as a type parameter, for example when
/// a `Vec<T>` need to be passed to serde.
#[derive(Clone, PartialEq)]
pub struct Serde<T>(pub T)
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize;

impl<T> Serde<T>
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    /// Consumes this wrapper, returning the inner value.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Serde<T>
where
    T: fmt::Debug,
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl<T> Deref for Serde<T>
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Serde<T>
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: PartialEq> PartialEq<T> for Serde<T>
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<'b, T> Deserialize<'b> for Serde<T>
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'b>,
    {
        De::deserialize(deserializer).map(De::into_inner).map(Serde)
    }
}

impl<T> Serialize for Serde<T>
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ser::new(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for De<ContentType> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize(deserializer)
            .map(|v: mime::Mime| ContentType::from(v))
            .map(De::new)
    }
}

impl<'a> Serialize for Ser<'a, ContentType> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(&mime::Mime::from(self.v.clone()), serializer)
    }
}

impl<'de> Deserialize<'de> for De<Cookie<'static>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CookieVisitor;

        impl<'de> Visitor<'de> for CookieVisitor {
            type Value = De<Cookie<'static>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an HTTP cookie header value")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Cookie::parse(v)
                    .map(Cookie::into_owned)
                    .map(De::new)
                    .map_err(|e| E::custom(format!("{:?}", e)))
            }
        }

        deserializer.deserialize_string(CookieVisitor)
    }
}

impl<'a, 'cookie> Serialize for Ser<'a, Cookie<'cookie>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.v.to_string())
    }
}

impl<'de> Deserialize<'de> for De<HeaderMap> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HeadersVisitor;

        impl<'de> Visitor<'de> for HeadersVisitor {
            type Value = De<HeaderMap>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a map from header names to header values")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(De::new(HeaderMap::new()))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut headers = HeaderMap::new();
                while let Some((k, values)) = visitor.next_entry::<String, Value>()? {
                    for v in values.0.iter() {
                        headers.append(
                            HeaderName::from_str(&k).map_err(V::Error::custom)?,
                            HeaderValue::from_bytes(v).map_err(V::Error::custom)?,
                        );
                    }
                }
                Ok(De::new(headers))
            }
        }

        struct Value(Vec<Vec<u8>>);

        impl<'de> Deserialize<'de> for Value {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_seq(ValueVisitor)
            }
        }

        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array of strings and sequences of bytes")
            }

            fn visit_unit<E>(self) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value(vec![]))
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                // Clamp to not OOM on rogue values.
                let capacity = cmp::min(visitor.size_hint().unwrap_or(0), 64);
                let mut values = Vec::with_capacity(capacity);
                while let Some(v) = visitor.next_element::<ByteBuf>()? {
                    values.push(v.into_vec());
                }
                Ok(Value(values))
            }
        }

        deserializer.deserialize_map(HeadersVisitor)
    }
}

impl<'a> Serialize for Ser<'a, HeaderMap> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct Value<'headers>(&'headers [Vec<u8>], bool);

        impl<'headers> Serialize for Value<'headers> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut serializer = serializer.serialize_seq(Some(self.0.len()))?;
                for v in self.0 {
                    if self.1 {
                        if let Ok(v) = str::from_utf8(v) {
                            serializer.serialize_element(v)?;
                            continue;
                        }
                    }
                    serializer.serialize_element(&Bytes::new(v))?;
                }
                serializer.end()
            }
        }

        let mut serializer = serializer.serialize_map(Some(self.v.keys_len()))?;
        for name in self.v.keys() {
            let values = self.v.get_all(name);
            serializer.serialize_entry(
                name.as_str(),
                &Value(
                    &values
                        .iter()
                        .map(|v| v.as_bytes().to_vec())
                        .collect::<Vec<Vec<u8>>>(),
                    self.pretty,
                ),
            )?;
        }
        serializer.end()
    }
}

impl<'de> Deserialize<'de> for De<Method> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MethodVisitor;

        impl<'de> Visitor<'de> for MethodVisitor {
            type Value = De<Method>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an HTTP method")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                v.parse::<Method>().map(De::new).map_err(E::custom)
            }
        }

        deserializer.deserialize_string(MethodVisitor)
    }
}

impl<'a> Serialize for Ser<'a, Method> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(self.v.as_ref(), serializer)
    }
}

impl<'de> Deserialize<'de> for De<Mime> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MimeVisitor;

        impl<'de> Visitor<'de> for MimeVisitor {
            type Value = De<Mime>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a mime type")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                v.parse::<Mime>()
                    .map(De::new)
                    .map_err(|_| E::custom("could not parse mime type"))
            }
        }

        deserializer.deserialize_string(MimeVisitor)
    }
}

impl<'a> Serialize for Ser<'a, Mime> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.v.as_ref())
    }
}

impl<'de> Deserialize<'de> for De<StatusCode> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = Deserialize::deserialize(deserializer)?;
        Ok(De::new(
            StatusCode::from_u16(code).map_err(D::Error::custom)?,
        ))
    }
}

impl<'a> Serialize for Ser<'a, StatusCode> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.v.as_u16().serialize(serializer)
    }
}

impl<'a> Serialize for Ser<'a, (StatusCode, String)> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_seq(Some(2))?;
        serializer.serialize_element(&Ser::new(&self.v.0))?;
        serializer.serialize_element(&self.v.1)?;
        serializer.end()
    }
}

impl<'de> Deserialize<'de> for De<(StatusCode, String)> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(De::new(deserializer.deserialize_seq(StatusVisitor)?))
    }
}

struct StatusVisitor;

impl<'de> Visitor<'de> for StatusVisitor {
    type Value = (StatusCode, String);

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "an array containing a status code and a reason string"
        )
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let code = visitor
            .next_element::<u16>()?
            .ok_or_else(|| V::Error::custom("Can't find the status code"))?;
        let code = StatusCode::from_u16(code).map_err(V::Error::custom)?;
        let reason = visitor
            .next_element::<String>()?
            .ok_or_else(|| V::Error::custom("Can't find the reason string"))?;
        Ok((code, reason))
    }
}

impl<'de> Deserialize<'de> for De<Tm> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TmVisitor;

        impl<'de> Visitor<'de> for TmVisitor {
            type Value = De<Tm>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a date and time according to RFC 3339")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                strptime(v, "%Y-%m-%dT%H:%M:%SZ")
                    .map(De::new)
                    .map_err(|e| E::custom(e.to_string()))
            }
        }

        deserializer.deserialize_string(TmVisitor)
    }
}

impl<'a> Serialize for Ser<'a, Tm> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.v.rfc3339().to_string())
    }
}

impl<'de> Deserialize<'de> for De<Uri> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UriVisitor;

        impl<'de> Visitor<'de> for UriVisitor {
            type Value = De<Uri>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an HTTP Uri value")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Uri::from_str(v)
                    .map(De::new)
                    .map_err(|e| E::custom(format!("{}", e)))
            }
        }

        deserializer.deserialize_string(UriVisitor)
    }
}

impl<'a> Serialize for Ser<'a, Uri> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // As of hyper 0.12, hyper::Uri (re-exported http::Uri)
        // does not implement as_ref due to underlying implementation
        // so we must construct a string to serialize it
        serializer.serialize_str(&self.v.to_string())
    }
}
