use std::time::Duration;

use ruma_common::serde::duration::ms::{deserialize as deserialize_ms, serialize as serialize_ms};
use serde::{Deserializer, Serializer, de::Error as _, ser::Error as _};

pub(super) fn serialize<S: Serializer>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
    if value.as_millis() == 0 {
        return Err(S::Error::custom("delay must be positive"));
    }

    serialize_ms(value, serializer)
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Duration, D::Error> {
    let value = deserialize_ms(deserializer)?;

    (!value.is_zero()).then_some(value).ok_or_else(|| D::Error::custom("delay must be positive"))
}
