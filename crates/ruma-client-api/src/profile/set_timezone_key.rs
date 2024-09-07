//! `PUT /_matrix/client/*/profile/{userId}/m.tz`
//!
//! Set the timezone key of the user.

/// Set the timezone key of the user.
pub mod unstable {
    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/us.cloke.msc4175.tz",
            1.12 => "/_matrix/client/v3/profile/{user_id}/m.tz",
        }
    }

    /// Request type for the `set_timezone_key` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose timezone will be set.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// [MSC4175][msc]: `m.tz` field for specifying a timezone the user is in
        ///
        /// [msc]: https://github.com/matrix-org/matrix-spec-proposals/blob/clokep/profile-tz/proposals/4175-profile-field-time-zone.md
        ///
        /// TODO: strong type this to be a valid IANA timezone?
        #[serde(rename = "us.cloke.msc4175.tz", skip_serializing_if = "Option::is_none")]
        pub tz: Option<String>,
    }

    /// Response type for the `set_timezone_key` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID and timezone.
        pub fn new(user_id: OwnedUserId, tz: Option<String>) -> Self {
            Self { user_id, tz }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
