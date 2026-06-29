//! `GET /_matrix/client/unstable/login/sso/callback`

pub mod unstable {
    //! `/unstable/`

    use std::borrow::Cow;

    use http::header::{LOCATION, SET_COOKIE};
    use ruma_common::{
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            unstable => "/_matrix/client/unstable/login/sso/callback/{idp_id}",
        }
    }

    /// Request type for the `sso_callback` endpoint.
    #[request]
    pub struct Request {
        /// Identity Provider ID
        #[ruma_api(path)]
        pub idp_id: String,

        /// Callback code
        #[ruma_api(query)]
        #[serde(default)]
        pub code: Option<String>,

        /// Callback state
        #[ruma_api(query)]
        #[serde(default)]
        pub state: Option<String>,
    }

    /// Response type for the `sso_callback` endpoint.
    #[response(status = FOUND)]
    pub struct Response {
        /// Redirect URL to the SSO identity provider.
        #[ruma_api(header = LOCATION)]
        pub location: String,

        /// Cookie storing state to secure the SSO process.
        #[ruma_api(header = SET_COOKIE)]
        pub cookie: Option<Cow<'static, str>>,
    }
}
