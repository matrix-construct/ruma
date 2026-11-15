//! `POST /_matrix/client/*/appservice/fed_proxy`
//!
//! Ask the homeserver to sign and send a federation request on behalf of the application service.

pub mod unstable {
    //! `/unstable/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4512

    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{
        OwnedServerName,
        api::{auth_scheme::AppserviceToken, request, response},
        metadata,
        serde::{JsonObject, Raw},
    };

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AppserviceToken,
        history: {
            unstable("io.element.msc4512") => "/_matrix/client/unstable/io.element.msc4512/appservice/fed_proxy",
        }
    }

    /// Request type for the `fed_proxy` endpoint.
    #[request]
    pub struct Request {
        /// The server to send the federation request to.
        pub destination: OwnedServerName,

        /// The HTTP method of the federation request.
        ///
        /// One of `GET`, `POST`, `PUT` and `DELETE`.
        pub method: String,

        /// The path of the federation request.
        ///
        /// It must lie under `/_matrix/federation/` and the application service's proxy prefix.
        pub path: String,

        /// The query parameters of the federation request.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub query: BTreeMap<String, String>,

        /// The JSON body of the federation request.
        ///
        /// Must be absent when `method` is `GET` or `DELETE`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub body: Option<Raw<JsonObject>>,
    }

    /// Response type for the `fed_proxy` endpoint.
    #[response]
    pub struct Response {
        /// The HTTP status of the federation response.
        pub status: UInt,

        /// The body of the federation response, when it is a JSON object.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<Raw<JsonObject>>,
    }

    impl Request {
        /// Creates a new `Request` with the given destination, method and path.
        pub fn new(destination: OwnedServerName, method: String, path: String) -> Self {
            Self { destination, method, path, query: BTreeMap::new(), body: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given federation response status.
        pub fn new(status: UInt) -> Self {
            Self { status, content: None }
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod tests {
        use ruma_common::api::IncomingRequest as _;
        use serde_json::{json, to_vec as to_json_vec};

        use super::Request;

        #[test]
        fn deserialization() {
            let body = json!({
                "destination": "remote.example.org",
                "method": "POST",
                "path": "/_matrix/federation/v1/rtc/livekit/get_token",
                "body": { "room_id": "!room:example.org" },
            });

            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::POST)
                    .uri("https://example.org/_matrix/client/unstable/io.element.msc4512/appservice/fed_proxy")
                    .body(to_json_vec(&body).unwrap())
                    .unwrap(),
                &[] as &[String],
            )
            .unwrap();

            assert_eq!(req.destination, "remote.example.org");
            assert_eq!(req.method, "POST");
            assert_eq!(req.path, "/_matrix/federation/v1/rtc/livekit/get_token");
            assert!(req.query.is_empty());
            assert_eq!(req.body.unwrap().json().get(), r#"{"room_id":"!room:example.org"}"#);
        }
    }
}
