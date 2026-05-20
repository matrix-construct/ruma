//! `GET /_synapse/admin/v1/register` (de-facto, Synapse vendor namespace)
//!
//! Issues a short-lived nonce to be consumed by the matching `POST` to
//! `/_synapse/admin/v1/register` (see [`register`](super::register)).
//!
//! This endpoint is not part of the Matrix client-server API specification.
//! It originates in Synapse and is implemented here for cross-implementation
//! parity with tooling that targets it (Element's Playwright suite, mautrix
//! bridges, ansible playbooks, etc.).

pub mod v1 {
    //! `/v1/`.

    use ruma_common::{
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_synapse/admin/v1/register",
        }
    }

    /// Request type for the `get_nonce` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_nonce` endpoint.
    #[response]
    pub struct Response {
        /// A nonce to be supplied in the body of a subsequent `POST` request.
        pub nonce: String,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given nonce.
        pub fn new(nonce: String) -> Self {
            Self { nonce }
        }
    }
}
