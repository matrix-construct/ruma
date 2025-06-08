//! `GET /_matrix/federation/*/event/{eventId}`
//!
//! Retrieves a single event.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1eventeventid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedServerName,
    };
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/event/{event_id}",
        }
    };

    /// Request type for the `get_event` endpoint.
    #[request]
    pub struct Request {
        /// The event ID to get.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,

        /// Query parameter to tell the server if it should return the redacted content of the
        /// requested event
        ///
        /// as per MSC2815: https://github.com/matrix-org/matrix-spec-proposals/pull/2815
        #[ruma_api(query)]
        #[serde(
            skip_serializing_if = "Option::is_none",
            alias = "fi.mau.msc2815.include_unredacted_content"
        )]
        pub include_unredacted_content: Option<bool>,
    }

    /// Response type for the `get_event` endpoint.
    #[response]
    pub struct Response {
        /// The `server_name` of the homeserver sending this transaction.
        pub origin: OwnedServerName,

        /// Time on originating homeserver when this transaction started.
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// The event.
        #[serde(rename = "pdus", with = "ruma_common::serde::single_element_seq")]
        pub pdu: Box<RawJsonValue>,
    }

    impl Request {
        /// Creates a new `Request` with the given event id.
        pub fn new(event_id: OwnedEventId, include_unredacted_content: Option<bool>) -> Self {
            Self { event_id, include_unredacted_content }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given server name, timestamp, and event.
        pub fn new(
            origin: OwnedServerName,
            origin_server_ts: MilliSecondsSinceUnixEpoch,
            pdu: Box<RawJsonValue>,
        ) -> Self {
            Self { origin, origin_server_ts, pdu }
        }
    }
}
