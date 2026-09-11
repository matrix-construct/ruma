//! `/v1/` and the final unstable alias defined by [MSC4140].
//!
//! Both paths require an access token identifying the delayed event's owner.
//!
//! [MSC4140]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

use ruma_common::{
    api::{auth_scheme::AccessToken, request, response},
    metadata,
};

use super::UpdateAction;

metadata! {
    method: POST,
    rate_limited: true,
    authentication: AccessToken,
    history: {
        unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events/{delay_id}/{action}",
        stable("org.matrix.msc4140.stable") => "/_matrix/client/v1/delayed_events/{delay_id}/{action}",
    }
}

/// Request to restart, send or cancel an authenticated delayed event.
///
/// The action is encoded in the path and the request has no body fields.
#[request]
pub struct Request {
    /// The delayed event to update.
    #[ruma_api(path)]
    pub delay_id: String,

    /// The requested action.
    #[ruma_api(path)]
    pub action: UpdateAction,
}

/// Empty response after a successful delayed event action.
///
/// A successful retry of the matching final action uses the same response.
#[response]
#[derive(Default)]
pub struct Response {}

impl Request {
    /// Creates an action request for the given delayed event.
    ///
    /// The same request type supports both the final unstable and stable routes.
    pub fn new(delay_id: String, action: UpdateAction) -> Self {
        Self { delay_id, action }
    }
}

impl Response {
    /// Creates an empty action response.
    ///
    /// Its JSON body is an empty object.
    pub fn new() -> Self {
        Self {}
    }
}
