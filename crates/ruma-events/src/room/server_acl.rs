//! Types for the [`m.room.server_acl`] event.
//!
//! [`m.room.server_acl`]: https://spec.matrix.org/latest/client-server-api/#mroomserver_acl

use ruma_common::ServerName;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use wildmatch::WildMatch;

use crate::EmptyStateKey;

/// The content of an `m.room.server_acl` event.
///
/// An event to indicate which servers are permitted to participate in the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.server_acl", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomServerAclEventContent {
    /// Whether to allow server names that are IP address literals.
    ///
    /// This is strongly recommended to be set to false as servers running with IP literal names
    /// are strongly discouraged in order to require legitimate homeservers to be backed by a
    /// valid registered domain name.
    #[serde(
        default = "ruma_common::serde::default_true",
        skip_serializing_if = "ruma_common::serde::is_true"
    )]
    pub allow_ip_literals: bool,

    /// The server names to allow in the room, excluding any port information.
    ///
    /// Wildcards may be used to cover a wider range of hosts, where `*` matches zero or more
    /// characters and `?` matches exactly one character.
    ///
    /// **Defaults to an empty list when not provided, effectively disallowing every server.**
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,

    /// The server names to disallow in the room, excluding any port information.
    ///
    /// Wildcards may be used to cover a wider range of hosts, where * matches zero or more
    /// characters and `?` matches exactly one character.
    ///
    /// Defaults to an empty list when not provided.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
}

impl RoomServerAclEventContent {
    /// Creates a new `RoomServerAclEventContent` with the given IP literal allowance flag, allowed
    /// and denied servers.
    #[inline]
    pub fn new(allow_ip_literals: bool, allow: Vec<String>, deny: Vec<String>) -> Self {
        Self { allow_ip_literals, allow, deny }
    }

    /// Returns true if and only if the server is allowed by the ACL rules.
    #[inline]
    pub fn is_allowed(&self, server_name: &ServerName) -> bool {
        if !self.allow_ip_literals && server_name.is_ip_literal() {
            return false;
        }

        let host = server_name.host();
        !self.deny_matches(host) && self.allow_matches(host)
    }

    /// Returns true if the input matches a pattern in the allow list specifically
    #[inline]
    pub fn allow_matches(&self, host: &str) -> bool {
        Self::matches(&self.allow, host)
    }

    /// Returns true if the input matches a pattern in the deny list specifically
    #[inline]
    pub fn deny_matches(&self, host: &str) -> bool {
        Self::matches(&self.deny, host)
    }

    /// Returns true if the input is equal to a string in the allow list specifically
    #[inline]
    pub fn allow_contains(&self, host: &str) -> bool {
        Self::contains(&self.allow, host)
    }

    /// Returns true if the input is equal to a string in the deny list specifically
    #[inline]
    pub fn deny_contains(&self, host: &str) -> bool {
        Self::contains(&self.deny, host)
    }

    /// Returns true if the allow list is empty
    #[inline]
    pub fn allow_is_empty(&self) -> bool {
        self.allow.is_empty()
    }

    /// Returns true if the deny list is empty
    #[inline]
    pub fn deny_is_empty(&self) -> bool {
        self.deny.is_empty()
    }

    fn matches(a: &[String], s: &str) -> bool {
        a.iter().map(String::as_str).any(|a| WildMatch::new(a).matches(s))
    }

    fn contains(a: &[String], s: &str) -> bool {
        a.iter().map(String::as_str).any(|a| a == s)
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::server_name;
    use serde_json::{from_value as from_json_value, json};

    use super::RoomServerAclEventContent;
    use crate::OriginalStateEvent;

    #[test]
    fn default_values() {
        let json_data = json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.server_acl"
        });

        let server_acl_event: OriginalStateEvent<RoomServerAclEventContent> =
            from_json_value(json_data).unwrap();

        assert!(server_acl_event.content.allow_ip_literals);
        assert_eq!(server_acl_event.content.allow.len(), 0);
        assert_eq!(server_acl_event.content.deny.len(), 0);
    }

    #[test]
    fn acl_ignores_port() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: true,
            allow: vec!["*".to_owned()],
            deny: vec!["1.1.1.1".to_owned()],
        };
        assert!(!acl_event.is_allowed(server_name!("1.1.1.1:8000")));
    }

    #[test]
    fn acl_allow_ip_literal() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: true,
            allow: vec!["*".to_owned()],
            deny: Vec::new(),
        };
        assert!(acl_event.is_allowed(server_name!("1.1.1.1")));
    }

    #[test]
    fn acl_deny_ip_literal() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["*".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("1.1.1.1")));
    }

    #[test]
    fn acl_deny() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["*".to_owned()],
            deny: vec!["matrix.org".to_owned()],
        };
        assert!(!acl_event.is_allowed(server_name!("matrix.org")));
        assert!(acl_event.is_allowed(server_name!("conduit.rs")));
    }

    #[test]
    fn acl_explicit_allow() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["conduit.rs".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("matrix.org")));
        assert!(acl_event.is_allowed(server_name!("conduit.rs")));
    }

    #[test]
    fn acl_explicit_glob_1() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["*.matrix.org".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("matrix.org")));
        assert!(acl_event.is_allowed(server_name!("server.matrix.org")));
    }

    #[test]
    fn acl_explicit_glob_2() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["matrix??.org".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("matrix1.org")));
        assert!(acl_event.is_allowed(server_name!("matrix02.org")));
    }

    #[test]
    fn acl_ipv6_glob() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: true,
            allow: vec!["[2001:db8:1234::1]".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("[2001:db8:1234::2]")));
        assert!(acl_event.is_allowed(server_name!("[2001:db8:1234::1]")));
    }
}
