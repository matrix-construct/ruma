//! `GET /_matrix/client/*/rooms/{roomId}/is_joined`
//!
//! Check whether a single user or server is joined to a room.

pub mod unstable {
    //! `/unstable/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4502

    use ruma_common::{
        OwnedRoomId, OwnedServerName, OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable("io.element.msc4502") => "/_matrix/client/unstable/io.element.msc4502/rooms/{room_id}/is_joined",
        }
    }

    /// Request type for the `is_joined` endpoint.
    ///
    /// Exactly one of `mxid` and `server_name` must be set.
    #[request]
    pub struct Request {
        /// The room to check the membership in.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The user whose membership to check.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub mxid: Option<OwnedUserId>,

        /// The server whose membership to check.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub server_name: Option<OwnedServerName>,
    }

    /// Response type for the `is_joined` endpoint.
    #[response]
    pub struct Response {
        /// Whether the user or server is joined to the room.
        ///
        /// This is also `false` when the server does not know the room or the subject.
        pub joined: bool,
    }

    impl Request {
        /// Creates a new `Request` checking the given user's membership in the given room.
        pub fn with_user(room_id: OwnedRoomId, mxid: OwnedUserId) -> Self {
            Self { room_id, mxid: Some(mxid), server_name: None }
        }

        /// Creates a new `Request` checking the given server's membership in the given room.
        pub fn with_server(room_id: OwnedRoomId, server_name: OwnedServerName) -> Self {
            Self { room_id, mxid: None, server_name: Some(server_name) }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given membership answer.
        pub fn new(joined: bool) -> Self {
            Self { joined }
        }
    }
}
