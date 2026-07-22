//! `POST /_matrix/app/*/keys/query`
//!
//! Queries device and cross-signing keys from an application service.

pub mod unstable {
    //! `/unstable/org.matrix.msc3984/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3984

    use std::collections::BTreeMap;

    use ruma_common::{
        OwnedDeviceId, OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        encryption::{CrossSigningKey, DeviceKeys},
        metadata,
        serde::Raw,
    };

    /// Device IDs to query for each user.
    pub type DeviceKeyQuery = BTreeMap<OwnedUserId, Vec<OwnedDeviceId>>;

    /// Device keys returned for each user and device.
    pub type DeviceKeyMap = BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, Raw<DeviceKeys>>>;

    /// Cross-signing keys returned for each user.
    pub type CrossSigningKeys = BTreeMap<OwnedUserId, Raw<CrossSigningKey>>;

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        path: "/_matrix/app/unstable/org.matrix.msc3984/keys/query",
    }

    /// Request type for the `query_keys` endpoint.
    #[request]
    pub struct Request {
        /// Device IDs to query for each user, where an empty list requests all devices.
        #[ruma_api(body)]
        pub device_keys: DeviceKeyQuery,
    }

    /// Response type for the `query_keys` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// Device keys for the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub device_keys: DeviceKeyMap,

        /// Master cross-signing keys for the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub master_keys: CrossSigningKeys,

        /// Self-signing keys for the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub self_signing_keys: CrossSigningKeys,

        /// User-signing keys for the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub user_signing_keys: CrossSigningKeys,
    }

    impl Request {
        /// Creates a new `Request` with the given device key query.
        pub fn new(device_keys: DeviceKeyQuery) -> Self {
            Self { device_keys }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self::default()
        }
    }
}
