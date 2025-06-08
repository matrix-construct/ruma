//! `POST /_matrix/client/*/rooms/{roomId}/report`
//!
//! Reports an abusive room.

pub mod v3 {
    //! `MSC4151` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4151

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4151/rooms/{room_id}/report",
            1.0 => "/_matrix/client/v3/rooms/{room_id}/report",
        }
    };

    /// Request type for the `report_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room ID being reported.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// Reason to report room.
        ///
        /// May be blank.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `report_room` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room ID and reason.
        pub fn new(room_id: OwnedRoomId, reason: Option<String>) -> Self {
            Self { room_id, reason }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
