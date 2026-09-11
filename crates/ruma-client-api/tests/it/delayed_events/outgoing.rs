use std::time::Duration;

use assign::assign;
use js_int::uint;
use ruma_client_api::delayed_events::{
    get_delayed_event::v1::Request as GetRequest,
    send_delayed_event::v3::Request as CreateRequest,
    update_delayed_event::{UpdateAction, v1::Request as UpdateRequest},
};
use ruma_common::{
    MilliSecondsSinceUnixEpoch,
    api::{FeatureFlag, auth_scheme::SendAccessToken, error::IntoHttpError},
    owned_room_id,
    serde::Raw,
};
use ruma_events::{StateKey, TimelineEventType};
use serde_json::{Value as JsonValue, from_slice as from_json_slice, json};

use super::outgoing;

#[test]
fn creation_selects_stable_or_unstable_with_one_body() {
    for (feature, prefix) in
        [(FeatureFlag::Msc4140, "unstable/org.matrix.msc4140"), (FeatureFlag::Msc4140Stable, "v3")]
    {
        let request =
            outgoing(request(None, 1000), feature, SendAccessToken::IfRequired("token")).unwrap();

        let expected = format!(
            "/_matrix/client/{prefix}/rooms/!room:example.org/delayed_event/m.room.topic/txn"
        );

        assert_eq!(request.uri().path(), expected);
        assert_eq!(request.method(), "PUT");
        assert_eq!(request.headers()["authorization"], "Bearer token");
        assert_eq!(
            from_json_slice::<JsonValue>(request.body()).unwrap(),
            json!({"delay_ms": 1000, "content": {"topic": "later"}})
        );
    }
}

fn request(state_key: Option<StateKey>, millis: u64) -> CreateRequest {
    let content = Raw::from_json_string(r#"{"topic":"later"}"#.to_owned()).unwrap();

    CreateRequest::new_raw(
        TimelineEventType::RoomTopic,
        owned_room_id!("!room:example.org"),
        "txn".into(),
        Duration::from_millis(millis),
        state_key,
        content,
    )
}

#[test]
fn empty_state_key_and_timestamp_remain_distinct_from_absence() {
    let request = assign!(request(Some("".into()), 1), {
        ts: Some(MilliSecondsSinceUnixEpoch(uint!(123))),
    });

    let request =
        outgoing(request, FeatureFlag::Msc4140Stable, SendAccessToken::IfRequired("token"))
            .unwrap();

    assert_eq!(request.uri().query(), Some("ts=123"));
    assert_eq!(
        from_json_slice::<JsonValue>(request.body()).unwrap(),
        json!({"delay_ms": 1, "state_key": "", "content": {"topic": "later"}})
    );
}

#[test]
fn action_and_lookup_select_the_transition_path() {
    for (feature, prefix) in
        [(FeatureFlag::Msc4140, "unstable/org.matrix.msc4140"), (FeatureFlag::Msc4140Stable, "v1")]
    {
        let action = UpdateRequest::new("delay".to_owned(), UpdateAction::Restart);
        let action =
            outgoing(action, feature.clone(), SendAccessToken::IfRequired("token")).unwrap();

        let lookup = GetRequest::new("delay".to_owned());
        let lookup = outgoing(lookup, feature, SendAccessToken::IfRequired("token")).unwrap();

        assert_eq!(
            action.uri().path(),
            format!("/_matrix/client/{prefix}/delayed_events/delay/restart")
        );

        assert_eq!(lookup.uri().path(), format!("/_matrix/client/{prefix}/delayed_events/delay"));
        assert_eq!(action.method(), "POST");
        assert_eq!(lookup.method(), "GET");
        assert_eq!(action.headers()["authorization"], "Bearer token");
        assert_eq!(lookup.headers()["authorization"], "Bearer token");
        assert_eq!(from_json_slice::<JsonValue>(action.body()).unwrap(), json!({}));
        assert!(lookup.body().is_empty());
    }
}

#[test]
fn every_final_operation_requires_a_token_on_both_paths() {
    for feature in [FeatureFlag::Msc4140, FeatureFlag::Msc4140Stable] {
        let create = outgoing(request(None, 1), feature.clone(), SendAccessToken::None);
        let update = UpdateRequest::new("delay".to_owned(), UpdateAction::Send);
        let update = outgoing(update, feature.clone(), SendAccessToken::None);
        let get = outgoing(GetRequest::new("delay".to_owned()), feature, SendAccessToken::None);

        for result in [create, update, get] {
            assert!(matches!(result, Err(IntoHttpError::Authentication(_))));
        }
    }
}

#[test]
fn creation_serialization_obeys_safe_integer_bounds() {
    for millis in [1, 9_007_199_254_740_991] {
        let request = outgoing(
            request(None, millis),
            FeatureFlag::Msc4140Stable,
            SendAccessToken::IfRequired("token"),
        )
        .unwrap();

        assert_eq!(from_json_slice::<JsonValue>(request.body()).unwrap()["delay_ms"], millis);
    }

    for millis in [0, 9_007_199_254_740_992, u64::MAX] {
        let result = outgoing(
            request(None, millis),
            FeatureFlag::Msc4140Stable,
            SendAccessToken::IfRequired("token"),
        );

        assert!(matches!(result, Err(IntoHttpError::Json(_))));
    }
}
