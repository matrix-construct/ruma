//! `/v1/` and the final unstable alias defined by [MSC4140].
//!
//! Both paths return the original content with an optional nested final outcome.
//!
//! [MSC4140]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

use std::time::Duration;

use ruma_common::{
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
    api::{auth_scheme::AccessToken, error::StandardErrorBody, request, response},
    metadata,
    serde::Raw,
};
use ruma_events::{AnyTimelineEventContent, StateKey, TimelineEventType};
use serde::{Deserialize, Serialize};

metadata! {
    method: GET,
    rate_limited: true,
    authentication: AccessToken,
    history: {
        unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events/{delay_id}",
        stable("org.matrix.msc4140.stable") => "/_matrix/client/v1/delayed_events/{delay_id}",
    }
}

/// Request to retrieve a delayed event owned by the authenticated user.
///
/// Stable and final unstable lookups share the same request shape.
#[request]
pub struct Request {
    /// The delayed event to retrieve.
    #[ruma_api(path)]
    pub delay_id: String,
}

/// The original delayed event and its optional final outcome.
///
/// Final results retain their original content and use the nested `finalised` object.
#[response]
pub struct Response {
    /// The delayed event information in the response body.
    #[ruma_api(body)]
    pub delayed_event: DelayedEventData,
}

/// The original delayed event and its optional final outcome.
///
/// Unknown fields are rejected, including the flattened outcome fields from the June interface.
/// Final outcomes use only the nested `finalised` object.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(deny_unknown_fields)]
pub struct DelayedEventData {
    /// The identifier of the delayed event.
    pub delay_id: String,

    /// The room in which the event was scheduled.
    pub room_id: OwnedRoomId,

    /// The type of the scheduled event.
    #[serde(rename = "type")]
    pub event_type: TimelineEventType,

    /// The state key, or none for a message event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<StateKey>,

    /// The positive delay from initial scheduling or the latest restart.
    #[serde(with = "crate::delayed_events::duration")]
    pub delay_ms: Duration,

    /// The timestamp of initial scheduling or the latest restart.
    pub delayed_since_ts: MilliSecondsSinceUnixEpoch,

    /// The original content object submitted for the event.
    #[serde(deserialize_with = "ruma_common::serde::deserialize_raw_object")]
    pub content: Raw<AnyTimelineEventContent>,

    /// The final outcome, absent while the event remains scheduled.
    #[serde(rename = "finalised", skip_serializing_if = "Option::is_none")]
    pub finalized: Option<Finalized>,
}

/// A final outcome with an unmassaged timestamp and mutually exclusive result fields.
///
/// A successful send carries an event identifier; an error carries a standard Matrix error.
/// Cancellation carries only its timestamp.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(untagged, deny_unknown_fields)]
pub enum Finalized {
    /// The event was sent successfully.
    Sent {
        /// The timestamp when the event was committed locally.
        #[serde(rename = "finalised_ts")]
        finalized_ts: MilliSecondsSinceUnixEpoch,

        /// The resulting event identifier.
        event_id: OwnedEventId,
    },

    /// Sending failed with a Matrix error.
    Error {
        /// The timestamp when the failure was finalized.
        #[serde(rename = "finalised_ts")]
        finalized_ts: MilliSecondsSinceUnixEpoch,

        /// The ordinary Matrix error that prevented sending.
        error: StandardErrorBody,
    },

    /// The delayed event was cancelled without a sending error.
    Canceled {
        /// The timestamp when cancellation was finalized.
        #[serde(rename = "finalised_ts")]
        finalized_ts: MilliSecondsSinceUnixEpoch,
    },
}

impl Request {
    /// Creates a lookup request for the given delayed event.
    ///
    /// The outgoing request requires authentication for both route spellings.
    pub fn new(delay_id: String) -> Self {
        Self { delay_id }
    }
}

impl Response {
    /// Creates a lookup response from delayed event information.
    ///
    /// The information is serialized directly as the response body.
    pub fn new(delayed_event: DelayedEventData) -> Self {
        Self { delayed_event }
    }
}

impl DelayedEventData {
    /// Creates pending delayed event information from the original content and schedule.
    ///
    /// Set `state_key` for a state event and `finalized` when a final outcome is available.
    pub fn new(
        delay_id: String,
        room_id: OwnedRoomId,
        event_type: TimelineEventType,
        content: Raw<AnyTimelineEventContent>,
        delay_ms: Duration,
        delayed_since_ts: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self {
            delay_id,
            room_id,
            event_type,
            state_key: None,
            delay_ms,
            delayed_since_ts,
            content,
            finalized: None,
        }
    }
}
