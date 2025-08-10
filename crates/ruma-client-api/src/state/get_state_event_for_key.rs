//! `GET /_matrix/client/*/rooms/{roomId}/state/{eventType}/{stateKey}`
//!
//! Get state events associated with a given key.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3roomsroomidstateeventtypestatekey

    use ruma_common::{
        OwnedRoomId,
        api::{Metadata, auth_scheme::AccessToken, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::{AnyStateEventContent, StateEventType};

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/state/{event_type}/{state_key}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/state/{event_type}/{state_key}",
        }
    }

    /// Request type for the `get_state_events_for_key` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request {
        /// The room to look up the state for.
        pub room_id: OwnedRoomId,

        /// The type of state to look up.
        pub event_type: StateEventType,

        /// The key of the state to look up.
        pub state_key: String,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID, event type and state key.
        pub fn new(room_id: OwnedRoomId, event_type: StateEventType, state_key: String) -> Self {
            Self { room_id, event_type, state_key }
        }
    }

    /// The format to use for the returned data.
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[derive(Default, Debug, PartialEq, Clone, Copy)]
    #[serde(rename_all = "lowercase")]
    pub enum StateEventFormat {
        /// Will return only the content of the state event.
        ///
        /// This is the default value if the format is unspecified in the request.
        #[default]
        Content,

        /// Will return the entire event in the usual format suitable for clients, including fields
        /// like event ID, sender and timestamp.
        Event,
    }

    /// Response type for the `get_state_events_for_key` endpoint, either the `Raw` `AnyStateEvent`
    /// or `AnyStateEventContent`.
    ///
    /// While it's possible to access the raw value directly, it's recommended you use the
    /// provided helper methods to access it, and `From` to create it.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The content of the state event.
        ///
        /// Since the inner type of the `Raw` does not implement `Deserialize`, you need to use
        /// `.deserialize_as_unchecked::<T>()` or
        /// `.cast_ref_unchecked::<T>().deserialize_with_type()` to deserialize it.
        #[ruma_api(body)]
        pub content: Raw<AnyStateEventContent>,
    }

    impl Response {
        /// Creates a new `Response` with the given content.
        pub fn new(content: Raw<AnyStateEventContent>) -> Self {
            Self { content }
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        fn try_into_http_request<T: Default + bytes::BufMut + AsRef<[u8]>>(
            self,
            base_url: &str,
            access_token: ruma_common::api::auth_scheme::SendAccessToken<'_>,
            considering: std::borrow::Cow<'_, ruma_common::api::SupportedVersions>,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use ruma_common::api::auth_scheme::AuthScheme;

            let mut http_request = http::Request::builder()
                .method(http::Method::GET)
                .uri(Self::make_endpoint_url(
                    considering,
                    base_url,
                    &[&self.room_id, &self.event_type, &self.state_key],
                    "",
                )?)
                .body(T::default())?;

            Self::Authentication::add_authentication(&mut http_request, access_token).map_err(
                |error| ruma_common::api::error::IntoHttpError::Authentication(error.into()),
            )?;

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            Self::check_request_method(request.method())?;

            // FIXME: find a way to make this if-else collapse with serde recognizing trailing
            // Option
            let (room_id, event_type, state_key): (OwnedRoomId, StateEventType, String) =
                if path_args.len() == 3 {
                    serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                        _,
                        serde::de::value::Error,
                    >::new(
                        path_args.iter().map(::std::convert::AsRef::as_ref),
                    ))?
                } else {
                    let (a, b) =
                        serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                            _,
                            serde::de::value::Error,
                        >::new(
                            path_args.iter().map(::std::convert::AsRef::as_ref),
                        ))?;

                    (a, b, "".into())
                };

            Ok(Self { room_id, event_type, state_key })
        }
    }
}
