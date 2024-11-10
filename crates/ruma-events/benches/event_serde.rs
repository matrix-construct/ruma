// `cargo bench` works, but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --features criterion --bench <name of the bench> -- --save-baseline <name>`.

#![allow(unused_imports, dead_code)]

use criterion::{criterion_group, criterion_main, Criterion};
use ruma_common::serde::Raw;
use ruma_events::{
    room::power_levels::RoomPowerLevelsEventContent, AnyStateEvent, AnyTimelineEvent,
    OriginalStateEvent,
};
use serde_json::json;

fn power_levels() -> serde_json::Value {
    json!({
        "content": {
            "ban": 50,
            "events": {
                "m.room.avatar": 50,
                "m.room.canonical_alias": 50,
                "m.room.history_visibility": 100,
                "m.room.name": 50,
                "m.room.power_levels": 100
            },
            "events_default": 0,
            "invite": 0,
            "kick": 50,
            "redact": 50,
            "state_default": 50,
            "users": {
                "@example:localhost": 100
            },
            "users_default": 0
        },
        "event_id": "$15139375512JaHAW:localhost",
        "origin_server_ts": 45,
        "sender": "@example:localhost",
        "room_id": "!room:localhost",
        "state_key": "",
        "type": "m.room.power_levels",
        "unsigned": {
            "age": 45
        }
    })
}

fn interpret_any_room_event(c: &mut Criterion) {
    let json_data = power_levels();

    c.bench_function("interpret `AnyTimelineEvent`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<AnyTimelineEvent>(json_data.clone()).unwrap();
        });
    });
}

fn deserialize_any_room_event(c: &mut Criterion) {
    let json_value = power_levels();
    let json_data = serde_json::to_vec(&json_value).unwrap();

    c.bench_function("deserialize `AnyTimelineEvent`", |b| {
        b.iter(|| {
            let _ = serde_json::from_slice::<AnyTimelineEvent>(&json_data).unwrap();
        });
    });
}

fn serialize_any_room_event(c: &mut Criterion) {
    let json_data = power_levels();

    c.bench_function("serialize `AnyTimelineEvent`", |b| {
        b.iter(|| {
            let _ = serde_json::to_vec(&json_data).unwrap();
        });
    });
}

fn interpret_any_state_event(c: &mut Criterion) {
    let json_data = power_levels();

    c.bench_function("interpret `AnyStateEvent`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<AnyStateEvent>(json_data.clone()).unwrap();
        });
    });
}

fn interpret_specific_event(c: &mut Criterion) {
    let json_data = power_levels();

    c.bench_function("interpret `OriginalStateEvent<PowerLevelsEventContent>`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<OriginalStateEvent<RoomPowerLevelsEventContent>>(
                json_data.clone(),
            )
            .unwrap();
        });
    });
}

fn deserialize_raw_specific_event(c: &mut Criterion) {
    let json_data = power_levels();
    let json_data: Raw<OriginalStateEvent<RoomPowerLevelsEventContent>> =
        serde_json::from_value(json_data).unwrap();

    c.bench_function("deserialize Raw<_> to `OriginalStateEvent<PowerLevelsEventContent>`", |b| {
        b.iter(|| {
            let _: OriginalStateEvent<RoomPowerLevelsEventContent> =
                json_data.clone().deserialize().unwrap();
        });
    });
}

criterion_group!(
    benches,
    interpret_any_room_event,
    serialize_any_room_event,
    deserialize_any_room_event,
    interpret_any_state_event,
    interpret_specific_event,
    deserialize_raw_specific_event,
);

criterion_main!(benches);
