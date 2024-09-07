//! `GET /_matrix/client/*/profile/{userId}/m.tz`
//!
//! Get the timezone key of a user.

/// Get the timezone key of a user.
pub mod unstable {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/us.cloke.msc4175.tz",
             1.12 => "/_matrix/client/v3/profile/{user_id}/m.tz",
        }
    };

    /// Request type for the `get_timezone` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose timezone will be retrieved.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `get_timezone` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// [MSC4175][msc]: `m.tz` field for specifying a timezone the user is in
        ///
        /// [msc]: https://github.com/matrix-org/matrix-spec-proposals/blob/clokep/profile-tz/proposals/4175-profile-field-time-zone.md
        ///
        /// TODO: strong type this to be a valid IANA timezone?
        #[serde(rename = "us.cloke.msc4175.tz", skip_serializing_if = "Option::is_none")]
        pub tz: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given timezone.
        pub fn new(tz: Option<String>) -> Self {
            Self { tz }
        }
    }
}
