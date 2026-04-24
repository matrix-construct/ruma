//! `GET /_tuwunel/remote_version/{server_name}`
//!
//! Probe a remote server's `/_matrix/federation/v1/version` endpoint and
//! return the response body together with the observed round-trip time.

pub mod unstable {
    //! Tuwunel-private unstable variant of the endpoint.

    use std::time::Duration;

    use ruma_common::{
        OwnedServerName,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };
    use serde_json::value::RawValue as RawJsonValue;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_tuwunel/remote_version/{server_name}",
        }
    }

    /// Request type for the `get_remote_version` endpoint.
    #[request]
    pub struct Request {
        /// Remote server to probe.
        #[ruma_api(path)]
        pub server_name: OwnedServerName,
    }

    /// Response type for the `get_remote_version` endpoint.
    #[response]
    pub struct Response {
        /// The remote server's `/_matrix/federation/v1/version` response body.
        pub data: Box<RawJsonValue>,

        /// Round-trip time of the probe, serialized as an integer
        /// millisecond count.
        #[serde(with = "ruma_common::serde::duration::ms")]
        pub rtt_ms: Duration,
    }

    impl Request {
        /// Creates a new `Request` with the given server name.
        pub fn new(server_name: OwnedServerName) -> Self {
            Self { server_name }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given data and rtt.
        pub fn new(data: Box<RawJsonValue>, rtt_ms: Duration) -> Self {
            Self { data, rtt_ms }
        }
    }
}
