#![allow(missing_docs)]
//! `PUT /_matrix/client/*/profile/{user_id}/{key_name}`
//!
//! Sets a custom profile key from the user

//! `msc4133` ([MSC])
//!
//! [MSC]: https://github.com/tcpipuk/matrix-spec-proposals/blob/main/proposals/4133-extended-profiles.md
pub mod unstable {
    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };
    use serde_json::Value as JsonValue;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/{key}",
            1.12 => "/_matrix/client/v3/profile/{user_id}/{key}",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request {
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        #[ruma_api(path)]
        pub key: String,

        #[serde(flatten)]
        #[ruma_api(body)]
        pub kv_pair: BTreeMap<String, JsonValue>,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        pub fn new(
            user_id: OwnedUserId,
            key: String,
            kv_pair: BTreeMap<String, JsonValue>,
        ) -> Self {
            Self { user_id, key, kv_pair }
        }
    }

    impl Response {
        pub fn new() -> Self {
            Self {}
        }
    }
}
