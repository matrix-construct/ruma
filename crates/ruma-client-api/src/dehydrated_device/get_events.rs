//! `GET /_matrix/client/*/dehydrated_device/{device_id}/events`
//!
//! Get to-device events for a dehydrated device.

pub mod unstable_v1;

pub mod unstable {
    //! `msc3814` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3814

    use ruma_common::{
        OwnedDeviceId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::AnyToDeviceEvent;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3814.v1/dehydrated_device/{device_id}/events",
        }
    }

    /// Request type for the `dehydrated_device/{device_id}/events` endpoint.
    #[request]
    pub struct Request {
        /// The unique ID of the device for which we would like to fetch events.
        #[ruma_api(path)]
        pub device_id: OwnedDeviceId,

        /// A point in time to continue getting events from.
        ///
        /// Should be a token from the `next_batch` field of a previous `/events`
        /// response.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<String>,
    }

    /// Response type for the `dehydrated_device/{device_id}/events` endpoint.
    #[response]
    pub struct Response {
        /// The batch token to supply in the `from` param of the next `/events` request.
        ///
        /// Omitted when there are no more events to retrieve. An empty `events` list does not
        /// mean that all events have been retrieved; only the absence of this field does.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,

        /// Messages sent directly between devices.
        pub events: Vec<Raw<AnyToDeviceEvent>>,
    }

    impl Request {
        /// Create a new request.
        pub fn new(device_id: OwnedDeviceId) -> Self {
            Self { device_id, from: None }
        }
    }

    impl Response {
        /// Create a new response with the given events.
        pub fn new(events: Vec<Raw<AnyToDeviceEvent>>) -> Self {
            Self { next_batch: None, events }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod client_tests {
        use std::borrow::Cow;

        use ruma_common::{
            api::{
                MatrixVersion, OutgoingRequestExt as _, SupportedVersions,
                auth_scheme::SendAccessToken,
            },
            owned_device_id,
        };

        use super::Request;

        #[test]
        fn serialize_request_from_query_param() {
            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_1].into(),
                features: Default::default(),
            };
            let mut request = Request::new(owned_device_id!("DEHYDRATED"));
            request.from = Some("s123".into());

            let req: http::Request<Vec<u8>> = request
                .try_into_http_request(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    Cow::Owned(supported),
                )
                .unwrap();

            assert_eq!(req.method(), http::Method::GET);
            assert_eq!(
                req.uri().path(),
                "/_matrix/client/unstable/org.matrix.msc3814.v1/dehydrated_device/DEHYDRATED/events"
            );
            assert_eq!(req.uri().query(), Some("from=s123"));
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {
        use ruma_common::api::{IncomingRequest as _, OutgoingResponse as _};

        use super::{Request, Response};

        #[test]
        fn deserialize_request_from_query_param() {
            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::GET)
                    .uri(
                        "https://matrix.org/_matrix/client/unstable/org.matrix.msc3814.v1\
                        /dehydrated_device/DEHYDRATED/events?from=s123",
                    )
                    .body(&[] as &[u8])
                    .unwrap(),
                &["DEHYDRATED"],
            )
            .unwrap();

            assert_eq!(req.device_id, "DEHYDRATED");
            assert_eq!(req.from.as_deref(), Some("s123"));
        }

        #[test]
        fn serialize_last_batch_without_next_batch() {
            let res = Response::new(vec![]).try_into_http_response::<Vec<u8>>().unwrap();

            assert_eq!(String::from_utf8_lossy(res.body()), r#"{"events":[]}"#);
        }
    }
}
