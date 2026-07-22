//! `POST /_matrix/app/*/keys/claim`
//!
//! Claims one-time keys from an application service.

pub mod unstable {
    //! `/unstable/org.matrix.msc3983/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3983

    use std::collections::BTreeMap;

    use ruma_common::{
        OneTimeKeyAlgorithm, OwnedDeviceId, OwnedOneTimeKeyId, OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        encryption::OneTimeKey,
        metadata,
        serde::Raw,
    };

    /// One-time key algorithms to claim for each user and device.
    pub type OneTimeKeyClaims =
        BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, Vec<OneTimeKeyAlgorithm>>>;

    /// One-time keys returned for each user and device.
    pub type OneTimeKeys = BTreeMap<
        OwnedUserId,
        BTreeMap<OwnedDeviceId, BTreeMap<OwnedOneTimeKeyId, Raw<OneTimeKey>>>,
    >;

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        path: "/_matrix/app/unstable/org.matrix.msc3983/keys/claim",
    }

    /// Request type for the `claim_keys` endpoint.
    #[request]
    pub struct Request {
        /// One-time key algorithms to claim for each user and device.
        #[ruma_api(body)]
        pub one_time_keys: OneTimeKeyClaims,
    }

    /// Response type for the `claim_keys` endpoint.
    #[response]
    pub struct Response {
        /// Claimed one-time keys for each user and device.
        #[ruma_api(body)]
        pub one_time_keys: OneTimeKeys,
    }

    impl Request {
        /// Creates a new `Request` with the given one-time key claims.
        pub fn new(one_time_keys: OneTimeKeyClaims) -> Self {
            Self { one_time_keys }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given one-time keys.
        pub fn new(one_time_keys: OneTimeKeys) -> Self {
            Self { one_time_keys }
        }
    }
}
