use ruma_client_api::delayed_events::get_delayed_event::v1::Finalized;
use serde_json::{
    Value as JsonValue, from_value as from_json_value, json, to_value as to_json_value,
};

#[test]
fn sent_error_and_canceled_outcomes_round_trip() {
    for (body, outcome) in [
        (json!({"finalised_ts": 101, "event_id": "$sent:example.org"}), "sent"),
        (
            json!({"finalised_ts": 102, "error": {"errcode": "M_FORBIDDEN", "error": "denied"}}),
            "error",
        ),
        (json!({"finalised_ts": 103}), "cancelled"),
    ] {
        let parsed = from_json_value::<Finalized>(body.clone()).unwrap();
        let actual = match &parsed {
            Finalized::Sent { .. } => "sent",
            Finalized::Error { .. } => "error",
            Finalized::Canceled { .. } => "cancelled",
            #[cfg(not(ruma_unstable_exhaustive_types))]
            _ => panic!("unexpected outcome"),
        };

        assert_eq!(actual, outcome);
        assert_eq!(to_json_value(parsed).unwrap(), body);
    }
}

#[test]
fn finalization_requires_one_outcome_and_its_timestamp() {
    for body in [
        json!({"finalised_ts": 100, "event_id": "$sent:example.org", "error": {"errcode": "M_FORBIDDEN", "error": "denied"}}),
        json!({"event_id": "$sent:example.org"}),
        json!({"error": {"errcode": "M_FORBIDDEN", "error": "denied"}}),
        json!({}),
        json!({"finalised_ts": -1}),
        json!({"finalised_ts": 0.5}),
        json!({"finalised_ts": 9_007_199_254_740_992_u64}),
        JsonValue::Null,
    ] {
        assert!(from_json_value::<Finalized>(body.clone()).is_err(), "accepted {body}");
    }
}
