#![allow(clippy::exhaustive_structs)]

use std::collections::BTreeMap;

use serde::{Serialize, ser::Serializer};
use smallstr::SmallString;

use super::CanonicalJsonValue;
use crate::serde::{JsonCastable, JsonObject};

/// The inner type of `CanonicalJsonValue::Object`.
pub type CanonicalJsonObject = BTreeMap<CanonicalJsonName, CanonicalJsonValue>;

/// Strong type for serializing slices of CanonicalJsonMember
#[derive(Clone, Debug, Default)]
pub struct CanonicalJsonMembers<'a, K>(pub &'a [CanonicalJsonMember<K>]);

/// Strong type for serializing slices of CanonicalJsonMember
#[derive(Clone, Debug, Default)]
pub struct CanonicalJsonMembersRef<'a, K>(pub &'a [CanonicalJsonMemberRef<'a, K>]);

/// Strong type for serializing slices of CanonicalJsonMemberOptional
#[derive(Clone, Debug, Default)]
pub struct CanonicalJsonMembersOptional<'a, K>(pub &'a [CanonicalJsonMemberOptional<K>]);

/// Strong type for serializing slices of CanonicalJsonMemberOptional
#[derive(Clone, Debug, Default)]
pub struct CanonicalJsonMembersRefOptional<'a, K>(pub &'a [CanonicalJsonMemberRefOptional<'a, K>]);

/// Inner type component for CanonicalJsonMembers with `AsRef<str>` keys and Into Values
#[allow(type_alias_bounds)]
pub type CanonicalJsonMember<K: AsRef<str>> = (K, CanonicalJsonValue);

/// Inner type component for CanonicalJsonMembers with `AsRef<str>` keys and Into Values
#[allow(type_alias_bounds)]
pub type CanonicalJsonMemberRef<'a, K: AsRef<str>> = (K, &'a CanonicalJsonValue);

/// Inner type component for CanonicalJsonMembers with `AsRef<str>` keys and Optional Into Values
#[allow(type_alias_bounds)]
pub type CanonicalJsonMemberOptional<K: AsRef<str>> = (K, Option<CanonicalJsonValue>);

/// Inner type component for CanonicalJsonMembers with `AsRef<str>` keys and Optional Into Values
#[allow(type_alias_bounds)]
pub type CanonicalJsonMemberRefOptional<'a, K: AsRef<str>> = (K, Option<&'a CanonicalJsonValue>);

/// Property name (or key) for an Object. This is a string-like but typographically distinct for
/// optimization purposes.
pub type CanonicalJsonName = SmallString<[u8; NAME_INLINE_CAP]>;

/// Opinionated buffer size of the CanonicalJsonName type.
const NAME_INLINE_CAP: usize = 32;

impl<T> JsonCastable<CanonicalJsonObject> for T where T: JsonCastable<JsonObject> {}

impl<'a, K> Serialize for CanonicalJsonMembers<'a, K>
where
    K: AsRef<str> + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        debug_assert!(
            self.0.iter().map(|(k, _)| k).map(AsRef::<str>::as_ref).is_sorted(),
            "Input members must be sorted to be canonical."
        );

        let len = self.0.len();
        serializer.serialize_map(Some(len)).and_then(|mut map| {
            self.0
                .iter()
                .try_for_each(|(k, v)| map.serialize_entry(k, v))
                .and_then(move |()| map.end())
        })
    }
}

impl<'a, K> Serialize for CanonicalJsonMembersRef<'a, K>
where
    K: AsRef<str> + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        debug_assert!(
            self.0.iter().map(|(k, _)| k).map(AsRef::<str>::as_ref).is_sorted(),
            "Input members must be sorted to be canonical."
        );

        let len = self.0.len();
        serializer.serialize_map(Some(len)).and_then(|mut map| {
            self.0
                .iter()
                .try_for_each(|(k, v)| map.serialize_entry(k, *v))
                .and_then(move |()| map.end())
        })
    }
}

impl<'a, K> Serialize for CanonicalJsonMembersOptional<'a, K>
where
    K: AsRef<str> + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        debug_assert!(
            self.0.iter().map(|(k, _)| k).map(AsRef::<str>::as_ref).is_sorted(),
            "Input members must be sorted to be canonical."
        );

        let len = self.0.iter().filter(|(_, v)| v.is_some()).count();
        serializer.serialize_map(Some(len)).and_then(|mut map| {
            self.0
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(move |v| (k, v)))
                .try_for_each(|(k, v)| map.serialize_entry(k, v))
                .and_then(move |()| map.end())
        })
    }
}

impl<'a, K> Serialize for CanonicalJsonMembersRefOptional<'a, K>
where
    K: AsRef<str> + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        debug_assert!(
            self.0.iter().map(|(k, _)| k).map(AsRef::<str>::as_ref).is_sorted(),
            "Input members must be sorted to be canonical."
        );

        let len = self.0.iter().filter(|(_, v)| v.is_some()).count();
        serializer.serialize_map(Some(len)).and_then(|mut map| {
            self.0
                .iter()
                .filter_map(|(k, v)| v.map(move |v| (k, v)))
                .try_for_each(|(k, v)| map.serialize_entry(k, v))
                .and_then(move |()| map.end())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_to_string() {
        const CANONICAL_STR: &str = r#"{"city":"London","street":"10 Downing Street"}"#;

        let members: [CanonicalJsonMember<_>; _] =
            [("city", "London".into()), ("street", "10 Downing Street".into())];

        let json = serde_json::to_string(&CanonicalJsonMembers(&members)).unwrap();
        assert_eq!(format!("{json}"), CANONICAL_STR);
    }

    #[test]
    fn remembers_to_string() {
        const CANONICAL_STR: &str = r#"{"city":"London","occupant":{"office":"prime minister"},"street":"10 Downing Street"}"#;
        const SUB_OBJECT: &str = r#"{"office":"prime minister"}"#;

        let sub_value: CanonicalJsonValue = serde_json::from_str(SUB_OBJECT).unwrap();
        let sub_raw = serde_json::value::to_raw_value(&sub_value).unwrap();
        let members: [CanonicalJsonMember<_>; _] = [
            ("city", "London".into()),
            ("occupant", sub_raw.into()),
            ("street", "10 Downing Street".into()),
        ];

        let json = serde_json::to_string(&CanonicalJsonMembers(&members)).unwrap();
        assert_eq!(format!("{json}"), CANONICAL_STR);
    }

    #[test]
    fn optional_members_to_string() {
        const CANONICAL_STR: &str = r#"{"city":"London","street":"10 Downing Street"}"#;

        let members: [CanonicalJsonMemberOptional<_>; _] = [
            ("city", Some("London".into())),
            ("occupant", None),
            ("street", Some("10 Downing Street".into())),
        ];

        let json = serde_json::to_string(&CanonicalJsonMembersOptional(&members)).unwrap();
        assert_eq!(format!("{json}"), CANONICAL_STR);
    }

    #[test]
    #[should_panic = "Input members must be sorted"]
    fn members_unsorted() {
        const CANONICAL_STR: &str = r#"{"city":"London","street":"10 Downing Street"}"#;

        let members: [CanonicalJsonMember<_>; _] =
            [("street", "10 Downing Street".into()), ("city", "London".into())];

        let json = serde_json::to_string(&CanonicalJsonMembers(&members)).unwrap();
        assert_eq!(format!("{json}"), CANONICAL_STR);
    }
}
