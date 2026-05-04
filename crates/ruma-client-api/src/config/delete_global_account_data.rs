//! `DELETE /_matrix/client/*/user/{userId}/account_data/{type}`
//!
//! Delete the global account data of a given type for a user.

pub mod unstable {
    //! `msc3391` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3391

    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };
    use ruma_events::GlobalAccountDataEventType;

    metadata! {
        method: DELETE,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3391/user/{user_id}/account_data/{event_type}",
        }
    }

    /// Request type for the `delete_global_account_data` endpoint.
    #[request]
    pub struct Request {
        /// The ID of the user to delete account_data for.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The event type of the account_data to delete.
        #[ruma_api(path)]
        pub event_type: GlobalAccountDataEventType,
    }

    /// Response type for the `delete_global_account_data` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID and event type.
        pub fn new(user_id: OwnedUserId, event_type: GlobalAccountDataEventType) -> Self {
            Self { user_id, event_type }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
