//! `GET /_matrix/client/*/rooms/{roomId}/event/{eventId}`
//!
//! Get a single event based on roomId/eventId

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.19/client-server-api/#get_matrixclientv3roomsroomideventeventid

    use ruma_common::{
        OwnedEventId, OwnedRoomId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::AnyTimelineEvent;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/event/{event_id}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/event/{event_id}",
        }
    }

    /// Request type for the `get_room_event` endpoint.
    #[request]
    pub struct Request {
        /// The ID of the room the event is in.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The ID of the event.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,

        /// Whether to respond with the original event, even if it has been
        /// redacted. [MSC2815].
        ///
        /// [MSC2815]: https://github.com/matrix-org/matrix-spec-proposals/pull/2815
        #[cfg(feature = "unstable-msc2815")]
        #[ruma_api(query)]
        #[serde(default, rename = "fi.mau.msc2815.include_unredacted_content")]
        pub include_unredacted_content: bool,
    }

    /// Response type for the `get_room_event` endpoint.
    #[response]
    pub struct Response {
        /// Arbitrary JSON of the event body.
        #[ruma_api(body)]
        pub event: Raw<AnyTimelineEvent>,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID and event ID.
        pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId) -> Self {
            Self {
                room_id,
                event_id,
                #[cfg(feature = "unstable-msc2815")]
                include_unredacted_content: false,
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event.
        pub fn new(event: Raw<AnyTimelineEvent>) -> Self {
            Self { event }
        }
    }
}
