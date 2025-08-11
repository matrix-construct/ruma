//! Types for the undocumented [`org.matrix.room.preview_urls`] event.
//!
//! [`org.matrix.room.preview_url`]: https://github.com/matrix-org/matrix-spec/issues/394

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::EmptyStateKey;

/// The content of an `org.matrix.room.preview_urls` event.
///
/// An event to indicate whether URL previews are disabled by default for the room or not.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.matrix.room.preview_urls", kind = State, state_key_type = EmptyStateKey)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomPreviewUrlsEventContent {
    /// Whether URL previews are disabled for the entire room
    #[serde(default)]
    pub disabled: bool,
}

impl RoomPreviewUrlsEventContent {
    /// Creates a new preview_url event with URL previews enabled by default (disabled: false)
    pub fn new(disabled: bool) -> Self {
        Self { disabled }
    }
}
