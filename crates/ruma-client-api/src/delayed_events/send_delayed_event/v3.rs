//! `/v3/` and the final unstable alias defined by [MSC4140].
//!
//! Both paths use the accepted request shape and require an access token.
//!
//! [MSC4140]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

use std::time::Duration;

use ruma_common::{
    MilliSecondsSinceUnixEpoch, OwnedRoomId, OwnedTransactionId,
    api::{auth_scheme::AccessToken, request, response},
    metadata,
    serde::Raw,
};
use ruma_events::{AnyTimelineEventContent, StateKey, TimelineEventType};

metadata! {
    method: PUT,
    rate_limited: true,
    authentication: AccessToken,
    history: {
        unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/rooms/{room_id}/delayed_event/{event_type}/{txn_id}",
        stable("org.matrix.msc4140.stable") => "/_matrix/client/v3/rooms/{room_id}/delayed_event/{event_type}/{txn_id}",
    }
}

/// Request to schedule a message or state event.
///
/// A missing state key selects a message; an empty state key still selects a state event.
#[request]
pub struct Request {
    /// The room to send the event to.
    #[ruma_api(path)]
    pub room_id: OwnedRoomId,

    /// The type of event to send.
    #[ruma_api(path)]
    pub event_type: TimelineEventType,

    /// The transaction identifier used to make scheduling idempotent.
    #[ruma_api(path)]
    pub txn_id: OwnedTransactionId,

    /// The positive delay before sending, in milliseconds on the wire.
    #[serde(with = "crate::delayed_events::duration")]
    pub delay_ms: Duration,

    /// The state key for a state event, including an empty key, or none for a message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<StateKey>,

    /// The original event content object.
    #[serde(deserialize_with = "ruma_common::serde::deserialize_raw_object")]
    pub content: Raw<AnyTimelineEventContent>,

    /// An application service's timestamp override for the eventual event.
    #[ruma_api(query)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,
}

/// Response containing the identifier of the accepted schedule.
///
/// The event identifier is unavailable until the schedule sends successfully.
#[response]
pub struct Response {
    /// The identifier used to manage or retrieve the delayed event.
    pub delay_id: String,
}

impl Request {
    /// Creates a request from typed event content.
    ///
    /// Returns an error if the content cannot be serialized.
    pub fn new(
        room_id: OwnedRoomId,
        txn_id: OwnedTransactionId,
        delay_ms: Duration,
        state_key: Option<StateKey>,
        content: &AnyTimelineEventContent,
    ) -> serde_json::Result<Self> {
        let content_raw = Raw::new(content)?;

        Ok(Self::new_raw(content.event_type(), room_id, txn_id, delay_ms, state_key, content_raw))
    }

    /// Creates a request from raw event content.
    ///
    /// The content must be an object and the delay must be positive when serialized.
    pub fn new_raw(
        event_type: TimelineEventType,
        room_id: OwnedRoomId,
        txn_id: OwnedTransactionId,
        delay_ms: Duration,
        state_key: Option<StateKey>,
        content: Raw<AnyTimelineEventContent>,
    ) -> Self {
        Self { room_id, event_type, txn_id, delay_ms, state_key, content, ts: None }
    }
}

impl Response {
    /// Creates a scheduling response with the accepted identifier.
    ///
    /// The identifier can subsequently be used for management and lookup.
    pub fn new(delay_id: String) -> Self {
        Self { delay_id }
    }
}
