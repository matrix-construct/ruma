use js_int::uint;
#[cfg(all(feature = "client", feature = "server"))]
use ruma_common::api::{IncomingResponse, OutgoingResponse};
use serde_json::{
    Value as JsonValue, from_value as from_json_value, json, to_value as to_json_value,
};

#[cfg(all(feature = "client", feature = "server"))]
use super::v3::Response;
use super::v3::{Capabilities, DelayedEventsCapability};

#[test]
fn delayed_capabilities_coexist() {
    let stable = DelayedEventsCapability::new(Some(uint!(86_400_000)), Some(uint!(100)));
    let unstable = DelayedEventsCapability::new(Some(uint!(60_000)), Some(uint!(20)));
    let capabilities = with_delayed_events(Some(stable), Some(unstable));
    let serialized = to_json_value(&capabilities).unwrap();

    assert_eq!(
        serialized["m.delayed_events"],
        json!({ "max_delay_ms": 86_400_000, "max_scheduled": 100 })
    );

    assert_eq!(
        serialized["org.matrix.msc4140.delayed_events"],
        json!({ "max_delay_ms": 60_000, "max_scheduled": 20 })
    );

    let decoded: Capabilities = from_json_value(serialized).unwrap();

    assert_eq!(decoded.delayed_events, Some(stable));
    assert_eq!(decoded.unstable_delayed_events, Some(unstable));
}

fn with_delayed_events(
    stable: Option<DelayedEventsCapability>,
    unstable: Option<DelayedEventsCapability>,
) -> Capabilities {
    let mut capabilities = Capabilities::new();

    capabilities.delayed_events = stable;
    capabilities.unstable_delayed_events = unstable;

    capabilities
}

#[cfg(all(feature = "client", feature = "server"))]
#[test]
fn delayed_capabilities_http_roundtrip() {
    let limits = DelayedEventsCapability::new(Some(uint!(86_400_000)), Some(uint!(100)));
    let response = Response::new(with_delayed_events(Some(limits), Some(limits)));
    let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();
    let decoded = Response::try_from_http_response(http_response).unwrap();

    assert_eq!(decoded.capabilities.delayed_events, Some(limits));
    assert_eq!(decoded.capabilities.unstable_delayed_events, Some(limits));
}

#[test]
fn absent_delayed_capabilities_stay_absent() {
    let capabilities = Capabilities::new();
    let serialized = to_json_value(&capabilities).unwrap();

    for name in ["m.delayed_events", "org.matrix.msc4140.delayed_events"] {
        assert!(capabilities.get(name).is_none());
        assert!(serialized.get(name).is_none());
    }
}

#[test]
fn delayed_capability_limits_are_independent() {
    for body in [
        json!({}),
        json!({ "max_delay_ms": 0 }),
        json!({ "max_scheduled": 0 }),
        json!({ "max_delay_ms": 9_007_199_254_740_991_u64, "max_scheduled": 9_007_199_254_740_991_u64 }),
    ] {
        let capability: DelayedEventsCapability = from_json_value(body.clone()).unwrap();

        assert_eq!(to_json_value(capability).unwrap(), body);
    }
}

#[test]
fn delayed_capability_get_and_set_use_typed_fields() {
    for name in ["m.delayed_events", "org.matrix.msc4140.delayed_events"] {
        let body = json!({ "max_delay_ms": 1000, "max_scheduled": 5 });
        let capabilities = with_capability(name, body.clone());
        let expected = DelayedEventsCapability::new(Some(uint!(1000)), Some(uint!(5)));

        assert_eq!(capabilities.get(name).unwrap().as_ref(), &body);
        assert_eq!(
            capabilities.delayed_events.or(capabilities.unstable_delayed_events),
            Some(expected)
        );

        assert_eq!(to_json_value(capabilities).unwrap()[name], body);
    }
}

fn with_capability(name: &str, value: JsonValue) -> Capabilities {
    let mut capabilities = Capabilities::new();

    capabilities.set(name, value).unwrap();

    capabilities
}

#[test]
fn delayed_capability_rejects_invalid_numeric_limits() {
    for field in ["max_delay_ms", "max_scheduled"] {
        for invalid in [json!(-1), json!(0.5), json!(9_007_199_254_740_992_u64), json!("1")] {
            let body = json!({ field: invalid });

            assert!(from_json_value::<DelayedEventsCapability>(body).is_err());
        }
    }
}
