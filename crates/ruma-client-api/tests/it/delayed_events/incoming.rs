use std::time::Duration;

use http::{Method, Request as HttpRequest};
use js_int::uint;
use ruma_client_api::delayed_events::{
    get_delayed_event::v1::Request as GetRequest,
    send_delayed_event::v3::Request as CreateRequest,
    update_delayed_event::{UpdateAction, v1::Request as UpdateRequest},
};
use ruma_common::{
    MilliSecondsSinceUnixEpoch,
    api::{IncomingRequest as _, error::FromHttpRequestError},
};
use serde_json::{Value as JsonValue, from_str as from_json_str, json};

#[test]
fn both_creation_paths_accept_the_final_body_and_massaged_timestamp() {
    for prefix in ["v3", "unstable/org.matrix.msc4140"] {
        let body = json!({"delay_ms": 1000, "state_key": "", "content": {"topic": "later"}});
        let uri = format!(
            "/_matrix/client/{prefix}/rooms/!room:example.org/delayed_event/m.room.topic/txn?ts=123"
        );
        let request = http_request(Method::PUT, &uri, body);
        let parsed = CreateRequest::try_from_http_request(
            request,
            &["!room:example.org", "m.room.topic", "txn"],
        )
        .unwrap();

        assert_eq!(parsed.delay_ms, Duration::from_millis(1000));
        assert_eq!(parsed.state_key.as_deref(), Some(""));
        assert_eq!(parsed.ts, Some(MilliSecondsSinceUnixEpoch(uint!(123))));
        assert_eq!(
            from_json_str::<JsonValue>(parsed.content.json().get()).unwrap(),
            json!({"topic": "later"})
        );
    }
}

fn http_request(method: Method, uri: &str, body: JsonValue) -> HttpRequest<String> {
    HttpRequest::builder().method(method).uri(uri).body(body.to_string()).unwrap()
}

#[test]
fn creation_accepts_positive_safe_integer_boundaries() {
    for millis in [1_u64, 9_007_199_254_740_991] {
        let parsed = parse(json!({"delay_ms": millis, "content": {}})).unwrap();

        assert_eq!(parsed.delay_ms, Duration::from_millis(millis));
        assert!(parsed.state_key.is_none());
        assert!(parsed.ts.is_none());
    }
}

fn parse(body: JsonValue) -> Result<CreateRequest, FromHttpRequestError> {
    let request = http_request(Method::PUT, "/", body);

    CreateRequest::try_from_http_request(request, &["!room:example.org", "m.room.topic", "txn"])
}

#[test]
fn creation_rejects_invalid_delay_and_content_types() {
    for value in [
        json!(0),
        json!(-1),
        json!(0.5),
        json!(9_007_199_254_740_992_u64),
        json!("1"),
        JsonValue::Null,
    ] {
        assert!(parse(json!({"delay_ms": value, "content": {}})).is_err());
    }

    for content in [json!([]), json!("text"), json!(42), json!(true), JsonValue::Null] {
        assert!(parse(json!({"delay_ms": 1, "content": content})).is_err());
    }

    assert!(parse(json!({"delay": 1, "content": {}})).is_err());
    assert!(parse(json!({"delay_ms": 1})).is_err());
    assert!(parse(json!({"delay_ms": 1, "state_key": 1, "content": {}})).is_err());
}

#[test]
fn management_reads_path_arguments_on_both_aliases() {
    for prefix in ["v1", "unstable/org.matrix.msc4140"] {
        let uri = format!("/_matrix/client/{prefix}/delayed_events/delay/cancel");
        let request = http_request(Method::POST, &uri, json!({}));
        let parsed = UpdateRequest::try_from_http_request(request, &["delay", "cancel"]).unwrap();
        let uri = format!("/_matrix/client/{prefix}/delayed_events/delay");
        let request = http_request(Method::GET, &uri, json!({}));
        let lookup = GetRequest::try_from_http_request(request, &["delay"]).unwrap();

        assert_eq!(parsed.delay_id, "delay");
        assert_eq!(parsed.action, UpdateAction::Cancel);
        assert_eq!(lookup.delay_id, "delay");
    }
}
