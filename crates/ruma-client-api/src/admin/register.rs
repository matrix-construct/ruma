//! `POST /_synapse/admin/v1/register` (de-facto, Synapse vendor namespace)
//!
//! Out-of-band account creation authenticated by HMAC-SHA1 over the
//! homeserver's pre-shared registration secret. Pairs with
//! [`get_nonce`](super::get_nonce); the nonce returned there is folded into
//! the MAC computed for the `POST` body.
//!
//! MAC formula:
//!
//! ```text
//! HMAC-SHA1(secret, nonce \0 username \0 password \0 admin|notadmin [\0 user_type])
//! ```
//!
//! hex-encoded and supplied in the `mac` field.
//!
//! This endpoint is not part of the Matrix client-server API specification.

pub mod v1 {
    //! `/v1/`.

    use std::time::Duration;

    use ruma_common::{
        OwnedDeviceId, OwnedServerName, OwnedUserId,
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_synapse/admin/v1/register",
        }
    }

    /// Request type for the `register` endpoint.
    #[request]
    pub struct Request {
        /// The nonce previously returned by `GET /_synapse/admin/v1/register`.
        pub nonce: String,

        /// Localpart of the user to create (or fully qualified Matrix ID).
        pub username: String,

        /// Display name to assign to the new account. Falls back to the
        /// localpart if omitted.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,

        /// Password to assign to the new account.
        pub password: String,

        /// Whether the new account is a server administrator.
        #[serde(default)]
        pub admin: bool,

        /// Optional Synapse-style user type (e.g. `support`, `bot`). Included
        /// in the MAC input when present.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub user_type: Option<String>,

        /// Hex-encoded HMAC-SHA1 over `nonce \0 username \0 password \0
        /// admin|notadmin [\0 user_type]`, keyed by the homeserver's
        /// registration shared secret.
        pub mac: String,

        /// If `true`, no `access_token`, `device_id`, `refresh_token`, or
        /// `expires_in_ms` are returned and no device is created.
        #[serde(default)]
        pub inhibit_login: bool,

        /// If `true`, a refresh token is issued alongside the access token.
        #[serde(default)]
        pub refresh_token: bool,

        /// Optional device ID to assign. A new one is generated otherwise.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub device_id: Option<OwnedDeviceId>,

        /// Optional initial display name for the newly created device.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub initial_device_display_name: Option<String>,
    }

    /// Response type for the `register` endpoint.
    #[response]
    pub struct Response {
        /// The fully qualified Matrix ID of the newly created account.
        pub user_id: OwnedUserId,

        /// `server_name` of the homeserver.
        pub home_server: OwnedServerName,

        /// Access token for the new device.
        ///
        /// Omitted if the request's `inhibit_login` was set to `true`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub access_token: Option<String>,

        /// ID of the device created alongside the account.
        ///
        /// Omitted if the request's `inhibit_login` was set to `true`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<OwnedDeviceId>,

        /// Refresh token for the new device. Present only when the request
        /// asked for one and `inhibit_login` was `false`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub refresh_token: Option<String>,

        /// Lifetime of the access token, in milliseconds.
        ///
        /// `None` indicates the access token will not expire.
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
            rename = "expires_in_ms"
        )]
        pub expires_in: Option<Duration>,
    }

    impl Request {
        /// Creates a new `Request` with required fields.
        pub fn new(nonce: String, username: String, password: String, mac: String) -> Self {
            Self {
                nonce,
                username,
                displayname: None,
                password,
                admin: false,
                user_type: None,
                mac,
                inhibit_login: false,
                refresh_token: false,
                device_id: None,
                initial_device_display_name: None,
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with required fields.
        pub fn new(user_id: OwnedUserId, home_server: OwnedServerName) -> Self {
            Self {
                user_id,
                home_server,
                access_token: None,
                device_id: None,
                refresh_token: None,
                expires_in: None,
            }
        }
    }
}
