use std::{
    clone::Clone,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::transmute,
    str,
};

use serde::{
    de::{self, Deserialize, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, Visitor},
    ser::{Serialize, Serializer},
};
use serde_json::value::{RawValue as RawJsonValue, Value as JsonValue};
use smallvec::SmallVec;

/// A wrapper around `Box<RawValue>` with a generic parameter for the expected
/// Rust type.
///
/// Ruma offers the `Raw` wrapper to enable passing around JSON text that is
/// only partially validated. This is useful when a client receives events that
/// do not follow the spec perfectly or a server needs to generate reference
/// hashes with the original canonical JSON string. All structs and enums
/// representing event types implement `Deserialize`, therefore they can be used
/// with `Raw`. Since `Raw` does not change the JSON string, it should be used
/// to pass around events in a lossless way.
///
/// ```no_run
/// # use serde::Deserialize;
/// # use ruma_common::serde::Raw;
/// # #[derive(Deserialize)]
/// # struct AnyTimelineEvent;
///
/// let json = r#"{ "type": "imagine a full event", "content": {...} }"#;
///
/// let deser = serde_json::from_str::<Raw<AnyTimelineEvent>>(json)
///     .unwrap() // the first Result from serde_json::from_str, will not fail
///     .deserialize() // deserialize to the inner type
///     .unwrap(); // finally get to the AnyTimelineEvent
/// ```
pub struct Raw<T, const INLINE_SIZE: usize = DEFAULT_INLINE_SIZE> {
    json: Inner<INLINE_SIZE>,
    _ev: PhantomData<T>,
}

/// Marker trait for restricting the types [`Raw::deserialize_as`],
/// [`Raw::cast`] and [`Raw::cast_ref`] can be called with.
///
/// Implementing this trait for a type `U` means that it is safe to cast from
/// `U` to `T` because `T` can be deserialized from the same JSON as `U`.
pub trait JsonCastable<T> {}

type Inner<const INLINE_SIZE: usize> = SmallVec<[u8; INLINE_SIZE]>;

const DEFAULT_INLINE_SIZE: usize = 48;

impl<T, const INLINE_SIZE: usize> Raw<T, INLINE_SIZE> {
    /// Create a `Raw` by serializing the given `T`.
    ///
    /// Shorthand for
    /// `serde_json::value::to_raw_value(val).map(Raw::from_json)`, but
    /// specialized to `T`.
    ///
    /// # Errors
    ///
    /// Fails if `T`s [`Serialize`] implementation fails.
    #[inline]
    pub fn new(val: &T) -> serde_json::Result<Self>
    where
        T: Serialize,
    {
        let mut s = Self { json: Inner::new(), _ev: PhantomData };
        s.write(val)?;
        Ok(s)
    }

    /// Copy a borrowed `str` of JSON data to `Raw<_>`.
    #[inline]
    pub fn from_raw_value(json: &RawJsonValue) -> Self {
        Self { json: Inner::from_slice(json.get().as_bytes()), _ev: PhantomData }
    }

    /// Create a `Raw` from a json Value.
    #[inline]
    pub fn from_json_value(json: &JsonValue) -> Self {
        let mut s = Self { json: Inner::new(), _ev: PhantomData };
        s.write(json).expect("JsonValue failed to serialize to Raw<T>");
        s
    }

    /// Convert an owned `String` of JSON data to `Raw<T>`.
    ///
    /// This function is equivalent to `serde_json::from_str::<Raw<T>>` except
    /// that an allocation and copy is avoided if both of the following are
    /// true:
    ///
    /// * the input has no leading or trailing whitespace, and
    /// * the input has capacity equal to its length.
    #[inline]
    pub fn from_json_string(json: String) -> serde_json::Result<Self> {
        let raw = RawJsonValue::from_string(json)?;
        Ok(Self::from_json(raw))
    }

    /// Create a `Raw` from a boxed `RawValue`.
    #[inline]
    pub fn from_json(json: Box<RawJsonValue>) -> Self {
        let len = json.get().len();
        let bs: Box<str> = json.into();
        let p: *mut u8 = Box::into_raw(bs).cast();
        let v = unsafe { Vec::<u8>::from_raw_parts(p, len, len) };
        Self { json: Inner::from_vec(v), _ev: PhantomData }
    }

    /// Convert `self` into the underlying json value.
    #[inline]
    pub fn into_json(self) -> Box<RawJsonValue> {
        let bu = self.json.into_boxed_slice();
        let bs = unsafe { str::from_boxed_utf8_unchecked(bu) };
        let s = String::from(bs);
        RawJsonValue::from_string(s).expect("Failed to convert Raw<T> to Box<RawJsonValue>")
    }

    /// Access the underlying json value.
    #[inline]
    pub fn json(&self) -> &RawJsonValue {
        let s = unsafe { str::from_utf8_unchecked(self.json.as_slice()) };
        unsafe { transmute::<&str, &RawJsonValue>(s) }
    }

    /// Try to deserialize the JSON as the expected type.
    #[inline]
    pub fn deserialize<'a>(&'a self) -> serde_json::Result<T>
    where
        T: Deserialize<'a>,
    {
        self.deserialize_as_unchecked::<T>()
    }

    /// Try to deserialize the JSON as a custom type.
    #[inline]
    pub fn deserialize_as<'a, U>(&'a self) -> serde_json::Result<U>
    where
        T: JsonCastable<U>,
        U: Deserialize<'a>,
    {
        self.deserialize_as_unchecked()
    }

    /// Same as [`deserialize_as`][Self::deserialize_as], but without the trait
    /// restriction.
    pub fn deserialize_as_unchecked<'a, U>(&'a self) -> serde_json::Result<U>
    where
        U: Deserialize<'a>,
    {
        let ret: U = serde_json::from_str(self.json().get())?;
        Ok(ret)
    }

    /// Turns `Raw<T>` into `Raw<U>` without changing the underlying JSON.
    ///
    /// This is useful for turning raw specific event types into raw event enum
    /// types.
    #[inline]
    pub fn cast<U>(self) -> Raw<U, INLINE_SIZE>
    where
        T: JsonCastable<U>,
    {
        self.cast_unchecked()
    }

    /// Turns `&Raw<T>` into `&Raw<U>` without changing the underlying JSON.
    ///
    /// This is useful for turning raw specific event types into raw event enum
    /// types.
    #[inline]
    pub fn cast_ref<U>(&self) -> &Raw<U, INLINE_SIZE>
    where
        T: JsonCastable<U>,
    {
        self.cast_ref_unchecked()
    }

    /// Same as [`cast`][Self::cast], but without the trait restriction.
    #[inline]
    pub fn cast_unchecked<U>(self) -> Raw<U, INLINE_SIZE> {
        Raw::<U, INLINE_SIZE>::from_json(self.into_json())
    }

    /// Same as [`cast_ref`][Self::cast_ref], but without the trait restriction.
    #[inline]
    pub fn cast_ref_unchecked<U>(&self) -> &Raw<U, INLINE_SIZE> {
        unsafe { transmute(self) }
    }

    fn write<U>(&mut self, value: &U) -> serde_json::Result<()>
    where
        U: Serialize + Sized,
    {
        serde_json::to_writer(&mut self.json, value)
    }
}

impl<T, const INLINE_SIZE: usize> Clone for Raw<T, INLINE_SIZE> {
    fn clone(&self) -> Self {
        Self { json: self.json.clone(), _ev: self._ev }
    }
}

impl<T, const INLINE_SIZE: usize> Debug for Raw<T, INLINE_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::any::type_name;
        f.debug_struct(&format!("Raw::<{}>", type_name::<T>()))
            .field("json", &str::from_utf8(&self.json).expect("invalid utf8"))
            .finish()
    }
}

impl<T, const INLINE_SIZE: usize> From<JsonValue> for Raw<T, INLINE_SIZE> {
    #[inline]
    fn from(other: JsonValue) -> Self {
        Self::from_json_value(&other)
    }
}

impl<T, const INLINE_SIZE: usize> From<Box<RawJsonValue>> for Raw<T, INLINE_SIZE> {
    #[inline]
    fn from(other: Box<RawJsonValue>) -> Self {
        Self::from_json(other)
    }
}

impl<'de, T, const INLINE_SIZE: usize> Deserialize<'de> for Raw<T, INLINE_SIZE> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Box::<RawJsonValue>::deserialize(deserializer).map(Self::from_json)
    }
}

impl<T, const INLINE_SIZE: usize> Serialize for Raw<T, INLINE_SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.json().serialize(serializer)
    }
}

impl<T, const INLINE_SIZE: usize> Raw<T, INLINE_SIZE> {
    /// Try to access a given field inside this `Raw`, assuming it contains an
    /// object.
    ///
    /// Returns `Err(_)` when the contained value is not an object, or the field
    /// exists but is fails to deserialize to the expected type.
    ///
    /// Returns `Ok(None)` when the field doesn't exist or is `null`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # type CustomMatrixEvent = ();
    /// # fn foo() -> serde_json::Result<()> {
    /// # let raw_event: ruma_common::serde::Raw<()> = todo!();
    /// if raw_event.get_field::<String>("type")?.as_deref() == Some("org.custom.matrix.event") {
    ///     let event = raw_event.deserialize_as_unchecked::<CustomMatrixEvent>()?;
    ///     // ...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_field<'a, U>(&'a self, field_name: &str) -> serde_json::Result<Option<U>>
    where
        U: Deserialize<'a>,
    {
        struct FieldVisitor<'b>(&'b str);

        impl Visitor<'_> for FieldVisitor<'_> {
            type Value = bool;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "`{}`", self.0)
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<bool, E>
            where
                E: de::Error,
            {
                Ok(value == self.0)
            }
        }

        struct Field<'b>(&'b str);

        impl<'de> DeserializeSeed<'de> for Field<'_> {
            type Value = bool;

            #[inline]
            fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_identifier(FieldVisitor(self.0))
            }
        }

        struct SingleFieldVisitor<'b, T> {
            field_name: &'b str,
            _phantom: PhantomData<T>,
        }

        impl<'b, T> SingleFieldVisitor<'b, T> {
            #[inline]
            fn new(field_name: &'b str) -> Self {
                Self { field_name, _phantom: PhantomData }
            }
        }

        impl<'de, T> Visitor<'de> for SingleFieldVisitor<'_, T>
        where
            T: Deserialize<'de>,
        {
            type Value = Option<T>;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string")
            }

            #[inline(never)]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut res = None;
                while let Some(is_right_field) = map.next_key_seed(Field(self.field_name))? {
                    if is_right_field {
                        res = Some(map.next_value()?);
                    } else {
                        map.next_value::<IgnoredAny>()?;
                    }
                }

                Ok(res)
            }
        }

        let mut deserializer = serde_json::Deserializer::from_str(self.json().get());
        deserializer.deserialize_map(SingleFieldVisitor::new(field_name))
    }
}

impl<T> JsonCastable<JsonValue> for T {}

#[cfg(test)]
mod tests {
    #![allow(unused)]

    use serde::Deserialize;
    use serde_json::{from_str as from_json_str, value::RawValue as RawJsonValue};

    use super::Raw;
    use crate::{CanonicalJsonObject, CanonicalJsonValue};

    #[test]
    fn get_field() -> serde_json::Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct A<'a> {
            #[serde(borrow)]
            b: Vec<&'a str>,
        }

        const OBJ: &str = r#"{ "a": { "b": [  "c"] }, "z": 5 }"#;
        let raw: Raw<()> = from_json_str(OBJ)?;

        assert_eq!(raw.get_field::<u8>("z")?, Some(5));
        assert_eq!(raw.get_field::<&RawJsonValue>("a")?.unwrap().get(), r#"{ "b": [  "c"] }"#);
        assert_eq!(raw.get_field::<A<'_>>("a")?, Some(A { b: vec!["c"] }));
        assert_eq!(raw.get_field::<u8>("b")?, None);
        raw.get_field::<u8>("a").unwrap_err();

        Ok(())
    }

    #[test]
    fn de_raw_struct() -> serde_json::Result<()> {
        #[derive(Clone, Deserialize, Debug)]
        struct A<'a> {
            foo: String,
            bar: CanonicalJsonObject,
            baz: &'a str,
        }

        const OBJ: &str = r#"{ "foo":"foo", "bar":{"aaa":"bbb"}, "baz": "baz" }"#;
        let a: Raw<A<'_>> = serde_json::from_str(OBJ).unwrap();

        assert_eq!(a.json().get(), OBJ);

        Ok(())
    }

    #[test]
    fn de_struct_with_raw() -> serde_json::Result<()> {
        #[derive(Clone, Deserialize, Debug)]
        struct A<'a> {
            foo: String,
            bar: Raw<CanonicalJsonObject>,
            baz: &'a str,
        }

        const OBJ: &str = r#"{ "foo":"foo", "bar":{"aaa":"bbb"}, "baz": "baz" }"#;
        let a: A<'_> = serde_json::from_str(OBJ).unwrap();

        assert_eq!(a.foo, "foo");
        assert_eq!(a.baz, "baz");
        assert_eq!(a.bar.json().get(), "{\"aaa\":\"bbb\"}");

        Ok(())
    }

    #[test]
    #[should_panic = "missing field `baz`"]
    fn de_struct_with_missing_field() {
        #[derive(Clone, Deserialize, Debug)]
        struct A<'a> {
            foo: String,
            bar: Raw<CanonicalJsonObject>,
            baz: &'a str,
        }

        const OBJ: &str = r#"{ "foo":"foo", "bar":{"aaa":"bbb"} }"#;
        let a: A<'_> = serde_json::from_str(OBJ).unwrap();

        assert_eq!(a.foo, "foo");
        assert_eq!(a.baz, "baz");
        assert_eq!(a.bar.json().get(), "{\"aaa\":\"bbb\"}");
    }

    #[test]
    #[should_panic = "invalid type"]
    fn de_struct_with_raw_bad_type() {
        #[derive(Clone, Deserialize, Debug)]
        struct A<'a> {
            foo: String,
            bar: Raw<CanonicalJsonObject>,
            baz: &'a str,
        }

        const OBJ: &str = r#"{ "foo":"foo", "bar":"{\"aaa\":\"bbb\"}", "baz": "baz" }"#;
        let a: A<'_> = serde_json::from_str(OBJ).unwrap();
        let b = a.bar.deserialize().unwrap();

        assert_eq!(a.foo, "foo");
        assert_eq!(a.baz, "baz");
        assert_eq!(a.bar.json().get(), "{\"aaa\":\"bbb\"}");
    }

    #[test]
    fn de_struct_with_raw_from_value() -> serde_json::Result<()> {
        #[derive(Clone, Deserialize, Debug)]
        struct A {
            foo: String,
            bar: Raw<CanonicalJsonObject>,
            baz: String,
        }

        const OBJ: &str = r#"{ "foo":"foo", "bar":{"aaa":"bbb"}, "baz": "baz" }"#;
        let object: CanonicalJsonObject = serde_json::from_str(OBJ).unwrap();
        let s = serde_json::to_string(&object).unwrap();
        let a: A = serde_json::from_str(&s).unwrap();

        assert_eq!(a.foo, "foo");
        assert_eq!(a.baz, "baz");
        assert_eq!(a.bar.json().get(), "{\"aaa\":\"bbb\"}");

        Ok(())
    }
}
