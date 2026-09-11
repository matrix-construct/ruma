use http::{Response, StatusCode};
use serde_json::{
    from_slice as from_json_slice, from_value as from_json_value, json, to_value as to_json_value,
};

use crate::api::{
    EndpointError, OutgoingResponse,
    error::{Error, ErrorBody, ErrorCode, ErrorKind, StandardErrorBody},
};

#[test]
fn excessive_delay_errors_preserve_each_wire_code() {
    for (kind, code) in [
        (ErrorKind::DelayTooLarge, "M_DELAY_TOO_LARGE"),
        (ErrorKind::UnstableDelayTooLarge, "ORG.MATRIX.MSC4140_DELAY_TOO_LARGE"),
    ] {
        let body = json!({ "errcode": code, "error": "Delay exceeds the server limit." });
        let deserialized: StandardErrorBody = from_json_value(body.clone()).unwrap();
        let errcode: ErrorCode = from_json_value(json!(code)).unwrap();

        assert_eq!(deserialized.kind, kind);
        assert_eq!(kind.errcode().as_str(), code);
        assert_eq!(errcode.as_str(), code);
        assert_eq!(to_json_value(deserialized).unwrap(), body);
        assert_eq!(to_json_value(kind).unwrap(), json!({ "errcode": code }));
    }
}

#[test]
fn excessive_delay_http_errors_preserve_status_and_body() {
    for (kind, code) in [
        (ErrorKind::DelayTooLarge, "M_DELAY_TOO_LARGE"),
        (ErrorKind::UnstableDelayTooLarge, "ORG.MATRIX.MSC4140_DELAY_TOO_LARGE"),
    ] {
        let body = StandardErrorBody::new(kind.clone(), "Delay exceeds the server limit.".into());
        let error = Error::new(StatusCode::BAD_REQUEST, ErrorBody::Standard(body));
        let response: Response<Vec<u8>> = error.try_into_http_response().unwrap();
        let expected = json!({ "errcode": code, "error": "Delay exceeds the server limit." });
        let decoded: StandardErrorBody = from_json_slice(response.body()).unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(to_json_value(decoded).unwrap(), expected);
        assert_eq!(Error::from_http_response(response).error_kind(), Some(&kind));
    }
}
