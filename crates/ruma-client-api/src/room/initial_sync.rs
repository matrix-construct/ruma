//! `GET /_matrix/client/*/rooms/{roomId}/initialSync`
//!
//! DEPRECATED

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3roomsroomidinitialsync

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedRoomId,
    };
    use ruma_events::{
        room::member::MembershipState, AnyRoomAccountDataEvent, AnyStateEvent, AnyTimelineEvent,
    };
    use serde::{Deserialize, Serialize};

    use crate::room::Visibility;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/initialSync",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/initialSync",
        }
    };

    /// Request type for the `get_room_event` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the room.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// Limit messages chunks size
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<usize>,
    }

    /// Response type for the `get_room_event` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The private data that this user has attached to this room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub account_data: Option<Vec<Raw<AnyRoomAccountDataEvent>>>,

        /// The user’s membership state in this room. One of: [invite, join, leave, ban].
        #[serde(skip_serializing_if = "Option::is_none")]
        pub membership: Option<MembershipState>,

        /// The pagination chunk for this room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub messages: Option<PaginationChunk>,

        /// The ID of this room.
        pub room_id: OwnedRoomId,

        /// If the user is a member of the room this will be the current state of the room as a
        /// list of events. If the user has left the room this will be the state of the room when
        /// they left it.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub state: Option<Vec<Raw<AnyStateEvent>>>,

        /// Whether this room is visible to the /publicRooms API or not.
        /// One of: [private, public].
        #[serde(skip_serializing_if = "Option::is_none")]
        pub visibility: Option<Visibility>,
    }

    /// Page of timeline events
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    pub struct PaginationChunk {
        /// If the user is a member of the room this will be a list of the most recent messages
        /// for this room. If the user has left the room this will be the messages that preceded
        /// them leaving. This array will consist of at most limit elements.
        pub chunk: Vec<Raw<AnyTimelineEvent>>,

        /// A token which correlates to the end of chunk. Can be passed to
        /// /rooms/<room_id>/messages to retrieve later events.
        pub end: String,

        /// A token which correlates to the start of chunk. Can be passed to
        /// /rooms/<room_id>/messages to retrieve earlier events. If no earlier events are
        /// available, this property may be omitted from the response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub start: Option<String>,
    }
}
