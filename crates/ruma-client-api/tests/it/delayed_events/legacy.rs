use std::time::Duration;

use http::{Request as HttpRequest, Response as HttpResponse};
use ruma_client_api::delayed_events::{
    DelayParameters,
    delayed_message_event::unstable::Request as MessageRequest,
    delayed_state_event::unstable::Request as StateRequest,
    get_all_delayed_events::unstable::Response as ListResponse,
    get_delayed_event::unstable::Response as GetResponse,
    send_delayed_event::unstable::Request as JuneRequest,
    update_delayed_event::{
        UpdateAction, unstable_v1::Request as BodyActionRequest,
        unstable_v2::Request as PathActionRequest,
    },
};
use ruma_common::{
    api::{
        FeatureFlag, IncomingResponse as _, OutgoingRequestExt as _, auth_scheme::SendAccessToken,
    },
    owned_room_id,
    serde::Raw,
};
use ruma_events::{MessageLikeEventType, StateEventType, TimelineEventType};
use serde_json::{Value as JsonValue, from_slice as from_json_slice, json};

use super::{outgoing, versions};

#[test]
fn june_creation_keeps_its_delay_field() {
    let content = Raw::from_json_string(r#"{"topic":"later"}"#.to_owned()).unwrap();
    let request = JuneRequest::new_raw(
        TimelineEventType::RoomTopic,
        owned_room_id!("!room:example.org"),
        "txn".into(),
        Duration::from_millis(1000),
        Some(String::new()),
        content,
    )
    .unwrap();

    let request =
        outgoing(request, FeatureFlag::Msc4140Stable, SendAccessToken::IfRequired("token"))
            .unwrap();

    assert!(request.uri().path().contains("/unstable/org.matrix.msc4140/"));
    assert_eq!(
        from_json_slice::<JsonValue>(request.body()).unwrap(),
        json!({"delay": 1000, "state_key": "", "content": {"topic": "later"}})
    );
}

#[test]
fn legacy_creation_keeps_query_delays_and_raw_content() {
    let delay = || DelayParameters::Timeout { timeout: Duration::from_millis(1000) };
    let content = || Raw::from_json_string("{}".to_owned()).unwrap();
    let message = MessageRequest::new_raw(
        owned_room_id!("!room:example.org"),
        "txn".into(),
        MessageLikeEventType::RoomMessage,
        delay(),
        content(),
    );

    let state = StateRequest::new_raw(
        owned_room_id!("!room:example.org"),
        String::new(),
        StateEventType::RoomTopic,
        delay(),
        Raw::from_json_string("{}".to_owned()).unwrap(),
    );

    let message =
        outgoing(message, FeatureFlag::Msc4140, SendAccessToken::IfRequired("token")).unwrap();

    let state =
        outgoing(state, FeatureFlag::Msc4140, SendAccessToken::IfRequired("token")).unwrap();

    assert_eq!(
        message.uri().path(),
        "/_matrix/client/v3/rooms/!room:example.org/send/m.room.message/txn"
    );
    assert_eq!(
        state.uri().path(),
        "/_matrix/client/v3/rooms/!room:example.org/state/m.room.topic/"
    );

    for request in [message, state] {
        assert_eq!(request.uri().query(), Some("org.matrix.msc4140.delay=1000"));
        assert_eq!(from_json_slice::<JsonValue>(request.body()).unwrap(), json!({}));
    }
}

#[test]
fn legacy_body_action_and_anonymous_path_action_survive() {
    let request = BodyActionRequest::new("delay".to_owned(), UpdateAction::Send);
    let body =
        outgoing(request, FeatureFlag::Msc4140, SendAccessToken::IfRequired("token")).unwrap();

    let request = PathActionRequest::new("delay".to_owned(), UpdateAction::Send);
    let path: HttpRequest<Vec<u8>> = request
        .try_into_http_request(
            "https://example.org",
            SendAccessToken::None,
            versions(FeatureFlag::Msc4140),
        )
        .unwrap();

    assert_eq!(
        body.uri().path(),
        "/_matrix/client/unstable/org.matrix.msc4140/delayed_events/delay"
    );
    assert_eq!(from_json_slice::<JsonValue>(body.body()).unwrap(), json!({"action": "send"}));
    assert!(path.uri().path().ends_with("/delay/send"));
    assert_eq!(from_json_slice::<JsonValue>(path.body()).unwrap(), json!({}));
    assert!(!path.headers().contains_key("authorization"));
}

#[test]
fn june_get_and_collection_keep_flat_finalization() {
    let body = json!({
        "delay_id": "delay", "room_id": "!room:example.org", "type": "m.room.topic",
        "content": {}, "delay": 1000, "running_since": 100,
        "event_id": "$sent:example.org", "finalised_ts": 1100,
    });

    let get = GetResponse::try_from_http_response(HttpResponse::new(body.to_string())).unwrap();
    let list = json!({"delayed_events": [body]});
    let list = ListResponse::try_from_http_response(HttpResponse::new(list.to_string())).unwrap();

    assert_eq!(get.delayed_event.delay, Duration::from_millis(1000));
    assert_eq!(get.delayed_event.running_since.0, 100_u32.into());
    assert_eq!(get.delayed_event.event_id.as_deref().unwrap().as_str(), "$sent:example.org");
    assert_eq!(get.delayed_event.finalized_ts.unwrap().0, 1100_u32.into());
    assert_eq!(list.delayed_events.len(), 1);
    assert_eq!(list.delayed_events[0].event_id, get.delayed_event.event_id);
}
