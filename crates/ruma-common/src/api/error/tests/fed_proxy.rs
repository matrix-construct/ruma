use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

use crate::api::error::{ErrorCode, ErrorKind, StandardErrorBody};

#[test]
fn fed_proxy_errors_preserve_each_wire_code() {
    for (kind, code) in [
        (
            ErrorKind::UnstableFedProxyConnectionFailed,
            "IO.ELEMENT.MSC4512_FEDPROXY_CONNECTION_FAILED",
        ),
        (
            ErrorKind::UnstableFedProxyDestinationDenied,
            "IO.ELEMENT.MSC4512_FEDPROXY_DESTINATION_DENIED",
        ),
        (ErrorKind::UnstableFedProxyPathNotAllowed, "IO.ELEMENT.MSC4512_FEDPROXY_PATH_NOT_ALLOWED"),
    ] {
        let body = json!({ "errcode": code, "error": "Federation proxy refused." });
        let deserialized: StandardErrorBody = from_json_value(body.clone()).unwrap();
        let errcode: ErrorCode = from_json_value(json!(code)).unwrap();

        assert_eq!(deserialized.kind, kind);
        assert_eq!(kind.errcode().as_str(), code);
        assert_eq!(errcode.as_str(), code);
        assert_eq!(to_json_value(deserialized).unwrap(), body);
        assert_eq!(to_json_value(kind).unwrap(), json!({ "errcode": code }));
    }
}
