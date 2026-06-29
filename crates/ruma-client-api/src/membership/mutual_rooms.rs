//! `GET /_matrix/client/*/mutual_rooms`
//!
//! Get the list of rooms a user shares with another user.

pub mod v1 {
    //! `/v1/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2666

    use js_int::UInt;
    use ruma_common::{
        OwnedRoomId, OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("uk.half-shot.msc2666.query_mutual_rooms") => "/_matrix/client/unstable/uk.half-shot.msc2666/user/mutual_rooms",
            1.19 => "/_matrix/client/v1/mutual_rooms",
        }
    }

    /// Request type for the `mutual_rooms` endpoint.
    #[request]
    pub struct Request {
        /// The user to search mutual rooms for.
        #[ruma_api(query)]
        pub user_id: OwnedUserId,

        /// The pagination token from a previous response, to get the next batch of rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat-empty-string-null",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        #[ruma_api(query)]
        pub from: Option<String>,
    }

    /// Response type for the `mutual_rooms` endpoint.
    #[response]
    pub struct Response {
        /// A list of rooms the user is in together with the authenticated user.
        pub joined: Vec<OwnedRoomId>,

        /// The total number of shared rooms, even when this response is batched.
        pub count: UInt,

        /// An opaque token, returned when the server paginates this response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, from: None }
        }

        /// Creates a new `Request` with the given user id, together with a pagination token.
        pub fn with_token(user_id: OwnedUserId, from: String) -> Self {
            Self { user_id, from: Some(from) }
        }
    }

    impl Response {
        /// Creates a `Response` with the given room ids and total count.
        pub fn new(joined: Vec<OwnedRoomId>, count: UInt) -> Self {
            Self { joined, count, next_batch: None }
        }

        /// Creates a `Response` with the given room ids, total count and pagination token.
        pub fn with_token(joined: Vec<OwnedRoomId>, count: UInt, next_batch: String) -> Self {
            Self { joined, count, next_batch: Some(next_batch) }
        }
    }
}
