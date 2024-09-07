//! `DELETE /_matrix/client/*/profile/{userId}/m.tz`
//!
//! Deletes the timezone key of the user.
#![allow(missing_docs)]

pub use v3 as unstable;
pub mod v3 {
    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/us.cloke.msc4175.tz",
            1.12 => "/_matrix/client/v3/profile/{user_id}/m.tz",
        }
    }

    /// Request type for the `delete_timezone_key` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose timezone will be deleted.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `delete_timezone_key` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
