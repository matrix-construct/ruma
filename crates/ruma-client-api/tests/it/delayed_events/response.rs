#[cfg(feature = "server")]
use std::time::Duration;

#[cfg(feature = "server")]
use assign::assign;
use http::Response as HttpResponse;
#[cfg(feature = "server")]
use js_int::uint;
use ruma_client_api::delayed_events::get_delayed_event::v1::{Finalized, Response};
#[cfg(feature = "server")]
use ruma_client_api::delayed_events::{
    get_delayed_event::v1::DelayedEventData, send_delayed_event::v3::Response as CreateResponse,
    update_delayed_event::v1::Response as UpdateResponse,
};
#[cfg(feature = "client")]
use ruma_common::api::IncomingResponse as _;
#[cfg(feature = "server")]
use ruma_common::{
    MilliSecondsSinceUnixEpoch, api::OutgoingResponse as _, owned_room_id, serde::Raw,
};
#[cfg(feature = "server")]
use ruma_events::TimelineEventType;
#[cfg(feature = "server")]
use serde_json::from_slice as from_json_slice;
use serde_json::{Value as JsonValue, from_value as from_json_value, json};

#[test]
#[cfg(feature = "client")]
fn lookup_deserializes_pending_and_nested_final_outcomes() {
    let pending = Response::try_from_http_response(HttpResponse::new(body().to_string())).unwrap();
    let pending = pending.delayed_event;

    assert!(pending.finalized.is_none());
    assert_eq!(pending.state_key.as_deref(), Some(""));
    assert_eq!(pending.delayed_since_ts.0, 100_u32.into());
    assert_eq!(pending.content.get_field::<String>("topic").unwrap().as_deref(), Some("later"));

    for outcome in outcomes() {
        let response = with_field(body(), "finalised", outcome.clone());
        let parsed =
            Response::try_from_http_response(HttpResponse::new(response.to_string())).unwrap();

        let expected = from_json_value::<Finalized>(outcome).unwrap();

        assert_eq!(json!(parsed.delayed_event.finalized.unwrap()), json!(expected));
    }
}

fn body() -> JsonValue {
    json!({"delay_id": "delay", "room_id": "!room:example.org", "type": "m.room.topic", "state_key": "", "content": {"topic": "later"}, "delay_ms": 1000, "delayed_since_ts": 100})
}

fn outcomes() -> impl Iterator<Item = JsonValue> {
    [
        json!({"finalised_ts": 1100, "event_id": "$sent:example.org"}),
        json!({"finalised_ts": 1100, "error": {"errcode": "M_FORBIDDEN", "error": "denied"}}),
        json!({"finalised_ts": 1100}),
    ]
    .into_iter()
}

fn with_field(mut body: JsonValue, field: &str, value: JsonValue) -> JsonValue {
    body[field] = value;
    body
}

#[test]
#[cfg(feature = "client")]
fn lookup_rejects_invalid_content_and_june_only_fields() {
    for content in [json!([]), json!(true), JsonValue::Null] {
        let response = with_field(body(), "content", content);

        assert!(Response::try_from_http_response(HttpResponse::new(response.to_string())).is_err());
    }

    let response = json!({"delay_id": "delay", "room_id": "!room:example.org", "type": "m.room.topic", "content": {}, "delay": 1, "running_since": 100});

    assert!(Response::try_from_http_response(HttpResponse::new(response.to_string())).is_err());
}

#[test]
#[cfg(feature = "client")]
fn lookup_rejects_mixed_june_fields() {
    let finalized = json!({"finalised_ts": 1100, "event_id": "$sent:example.org"});

    for base in [body(), with_field(body(), "finalised", finalized)] {
        for (field, value) in [
            ("delay", json!(1000)),
            ("running_since", json!(100)),
            ("event_id", json!("$sent:example.org")),
            ("error", json!({"errcode": "M_FORBIDDEN", "error": "denied"})),
            ("finalised_ts", json!(1100)),
        ] {
            let response = with_field(base.clone(), field, value);
            let parsed = Response::try_from_http_response(HttpResponse::new(response.to_string()));

            assert!(parsed.is_err(), "accepted June field {field}");
        }
    }
}

#[test]
#[cfg(feature = "client")]
fn lookup_rejects_unknown_final_fields() {
    let response = with_field(body(), "org.example.extension", json!({}));

    assert!(Response::try_from_http_response(HttpResponse::new(response.to_string())).is_err());
}

#[test]
#[cfg(feature = "server")]
fn lookup_serializes_only_the_final_contract() {
    for outcome in [None].into_iter().chain(outcomes().map(Some)) {
        let finalized = outcome.clone().map(|value| from_json_value::<Finalized>(value).unwrap());
        let response: HttpResponse<Vec<u8>> = response(finalized).try_into_http_response().unwrap();
        let expected = outcome.map_or_else(body, |value| with_field(body(), "finalised", value));

        assert_eq!(from_json_slice::<JsonValue>(response.body()).unwrap(), expected);
    }
}

#[cfg(feature = "server")]
fn response(finalized: Option<Finalized>) -> Response {
    let content = Raw::from_json_string(r#"{"topic":"later"}"#.to_owned()).unwrap();
    let event = DelayedEventData::new(
        "delay".to_owned(),
        owned_room_id!("!room:example.org"),
        TimelineEventType::RoomTopic,
        content,
        Duration::from_millis(1000),
        MilliSecondsSinceUnixEpoch(uint!(100)),
    );

    let event = assign!(event, { state_key: Some("".into()), finalized: finalized });

    Response::new(event)
}

#[test]
#[cfg(feature = "server")]
fn scheduling_and_action_responses_keep_the_protocol_envelopes() {
    let create: HttpResponse<Vec<u8>> =
        CreateResponse::new("delay".to_owned()).try_into_http_response().unwrap();

    let action: HttpResponse<Vec<u8>> = UpdateResponse::new().try_into_http_response().unwrap();

    assert_eq!(create.status(), 200);
    assert_eq!(from_json_slice::<JsonValue>(create.body()).unwrap(), json!({"delay_id": "delay"}));
    assert_eq!(action.status(), 200);
    assert_eq!(from_json_slice::<JsonValue>(action.body()).unwrap(), json!({}));
}
