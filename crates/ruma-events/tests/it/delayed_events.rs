use ruma_common::serde::{CanBeEmpty, Raw};
use ruma_events::{
    MessageLikeUnsigned, OriginalMessageLikeEvent, OriginalStateEvent, StateUnsigned,
    room::{
        member::{OriginalRoomMemberEvent, RoomMemberUnsigned},
        message::RoomMessageEventContent,
        name::{PossiblyRedactedRoomNameEventContent, RoomNameEventContent},
        redaction::{OriginalRoomRedactionEvent, RoomRedactionUnsigned},
    },
};
use serde::de::DeserializeOwned;
use serde_json::{
    Value as JsonValue, from_value as from_json_value, json, to_value as to_json_value,
};

type DelayIds<'a> = (Option<&'a str>, Option<&'a str>);

#[test]
fn message_unsigned_delay_ids() {
    check_unsigned(|unsigned: &MessageLikeUnsigned<RoomMessageEventContent>| {
        (unsigned.delay_id.as_deref(), unsigned.unstable_delay_id.as_deref())
    });
}

#[test]
fn state_unsigned_delay_ids() {
    check_unsigned(|unsigned: &StateUnsigned<PossiblyRedactedRoomNameEventContent>| {
        (unsigned.delay_id.as_deref(), unsigned.unstable_delay_id.as_deref())
    });
}

#[test]
fn member_unsigned_delay_ids() {
    check_unsigned(|unsigned: &RoomMemberUnsigned| {
        (unsigned.delay_id.as_deref(), unsigned.unstable_delay_id.as_deref())
    });
}

#[test]
fn redaction_unsigned_delay_ids() {
    check_unsigned(|unsigned: &RoomRedactionUnsigned| {
        (unsigned.delay_id.as_deref(), unsigned.unstable_delay_id.as_deref())
    });
}

fn check_unsigned<U>(ids: fn(&U) -> DelayIds<'_>)
where
    U: CanBeEmpty + Default + DeserializeOwned,
{
    let unsigned = U::default();

    assert!(unsigned.is_empty());
    assert_eq!(ids(&unsigned), (None, None));

    for (json, expected) in [
        (json!({}), (None, None)),
        (json!({ "delay_id": null, "org.matrix.msc4140.delay_id": null }), (None, None)),
        (json!({ "delay_id": "stable" }), (Some("stable"), None)),
        (json!({ "org.matrix.msc4140.delay_id": "unstable" }), (None, Some("unstable"))),
        (
            json!({ "delay_id": "stable", "org.matrix.msc4140.delay_id": "unstable" }),
            (Some("stable"), Some("unstable")),
        ),
    ] {
        let unsigned: U = from_json_value(json).unwrap();

        assert_eq!(ids(&unsigned), expected);
        assert_eq!(unsigned.is_empty(), expected == (None, None));
    }

    for json in [
        json!({ "delay_id": 42 }),
        json!({ "delay_id": [] }),
        json!({ "org.matrix.msc4140.delay_id": 42 }),
        json!({ "org.matrix.msc4140.delay_id": [] }),
    ] {
        assert!(from_json_value::<U>(json).is_err());
    }
}

#[test]
fn message_delay_ids_raw_roundtrip() {
    let event: OriginalMessageLikeEvent<RoomMessageEventContent> = roundtrip(json!({
        "type": "m.room.message",
        "content": { "msgtype": "m.text", "body": "Scheduled message" },
        "event_id": "$message",
        "room_id": "!room:example.org",
        "sender": "@alice:example.org",
        "origin_server_ts": 1000,
        "unsigned": { "delay_id": "stable", "org.matrix.msc4140.delay_id": "unstable" },
    }));

    assert_eq!(event.unsigned.delay_id.as_deref(), Some("stable"));
    assert_eq!(event.unsigned.unstable_delay_id.as_deref(), Some("unstable"));
}

#[test]
fn state_delay_ids_raw_roundtrip() {
    let event: OriginalStateEvent<RoomNameEventContent> = roundtrip(json!({
        "type": "m.room.name",
        "state_key": "",
        "content": { "name": "Scheduled name" },
        "event_id": "$state",
        "room_id": "!room:example.org",
        "sender": "@alice:example.org",
        "origin_server_ts": 1000,
        "unsigned": { "delay_id": "stable", "org.matrix.msc4140.delay_id": "unstable" },
    }));

    assert_eq!(event.unsigned.delay_id.as_deref(), Some("stable"));
    assert_eq!(event.unsigned.unstable_delay_id.as_deref(), Some("unstable"));
}

#[test]
fn member_delay_ids_raw_roundtrip() {
    let event: OriginalRoomMemberEvent = roundtrip(json!({
        "type": "m.room.member",
        "state_key": "@alice:example.org",
        "content": { "membership": "leave" },
        "event_id": "$member",
        "room_id": "!room:example.org",
        "sender": "@alice:example.org",
        "origin_server_ts": 1000,
        "unsigned": { "delay_id": "stable", "org.matrix.msc4140.delay_id": "unstable" },
    }));

    assert_eq!(event.unsigned.delay_id.as_deref(), Some("stable"));
    assert_eq!(event.unsigned.unstable_delay_id.as_deref(), Some("unstable"));
}

#[test]
fn redaction_delay_ids_raw_roundtrip() {
    let event: OriginalRoomRedactionEvent = roundtrip(json!({
        "type": "m.room.redaction",
        "content": { "redacts": "$original" },
        "event_id": "$redaction",
        "room_id": "!room:example.org",
        "sender": "@alice:example.org",
        "origin_server_ts": 1000,
        "unsigned": { "delay_id": "stable", "org.matrix.msc4140.delay_id": "unstable" },
    }));

    assert_eq!(event.unsigned.delay_id.as_deref(), Some("stable"));
    assert_eq!(event.unsigned.unstable_delay_id.as_deref(), Some("unstable"));
}

fn roundtrip<E: DeserializeOwned>(json: JsonValue) -> E {
    let raw: Raw<E> = from_json_value(json.clone()).unwrap();

    assert_eq!(to_json_value(&raw).unwrap(), json);
    raw.deserialize().unwrap()
}
