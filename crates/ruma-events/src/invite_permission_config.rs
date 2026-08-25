//! Types for the [`m.invite_permission_config`] account data.
//!
//! [`m.invite_permission_config`]: https://spec.matrix.org/v1.19/client-server-api/#minvite_permission_config

#[cfg(feature = "unstable-msc4155")]
use ruma_common::UserId;
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};
#[cfg(feature = "unstable-msc4155")]
use wildmatch::WildMatch;

use crate::PrivOwnedStr;

/// The content of an [`m.invite_permission_config`] account data.
///
/// Controls whether invites to this account are permitted.
///
/// [`m.invite_permission_config`]: https://spec.matrix.org/v1.19/client-server-api/#minvite_permission_config
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-msc4155"), derive(Default))]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    kind = GlobalAccountData,
    type = "m.invite_permission_config",
)]
pub struct InvitePermissionConfigEventContent {
    /// The default action chosen by the user that the homeserver should perform automatically when
    /// receiving an invitation for this account.
    ///
    /// A missing, invalid or unsupported value means that the user wants to receive invites as
    /// normal. Other parts of the specification might still have effects on invites, like
    /// [ignoring users].
    ///
    /// [ignoring users]: https://spec.matrix.org/v1.19/client-server-api/#ignoring-users
    #[serde(
        default,
        deserialize_with = "ruma_common::serde::default_on_error",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_action: Option<InvitePermissionAction>,

    /// Whether the [MSC4155] filtering rules below are active.
    ///
    /// Lets clients deactivate the rule lists without purging them. Defaults to `true`. Does not
    /// affect `default_action`.
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    #[cfg(feature = "unstable-msc4155")]
    #[serde(
        default = "ruma_common::serde::default_true",
        skip_serializing_if = "ruma_common::serde::is_true"
    )]
    pub enabled: bool,

    /// Globs matching users which are allowed to send an invite ([MSC4155]).
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    #[cfg(feature = "unstable-msc4155")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_users: Vec<String>,

    /// Globs matching users whose invites should be ignored ([MSC4155]).
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    #[cfg(feature = "unstable-msc4155")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_users: Vec<String>,

    /// Globs matching users whose invites should be blocked ([MSC4155]).
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    #[cfg(feature = "unstable-msc4155")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_users: Vec<String>,

    /// Globs matching servers whose users are allowed to send an invite ([MSC4155]).
    ///
    /// Matched against the server name of the inviting user after stripping any port suffix, like
    /// [server ACLs].
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    /// [server ACLs]: https://spec.matrix.org/v1.19/client-server-api/#mroomserver_acl
    #[cfg(feature = "unstable-msc4155")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_servers: Vec<String>,

    /// Globs matching servers whose users' invites should be ignored ([MSC4155]).
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    #[cfg(feature = "unstable-msc4155")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_servers: Vec<String>,

    /// Globs matching servers whose users' invites should be blocked ([MSC4155]).
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    #[cfg(feature = "unstable-msc4155")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_servers: Vec<String>,
}

impl InvitePermissionConfigEventContent {
    /// Creates a new empty `InvitePermissionConfigEventContent`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether this configuration filters nobody.
    ///
    /// A slot holding no blanket action and no rule the server would consult is inert, so a
    /// caller can skip evaluating it, and skip whatever it costs them to name a sender.
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    #[cfg(feature = "unstable-msc4155")]
    pub fn is_inert(&self) -> bool {
        self.default_action.is_none()
            && (!self.enabled
                || [
                    &self.allowed_users,
                    &self.ignored_users,
                    &self.blocked_users,
                    &self.allowed_servers,
                    &self.ignored_servers,
                    &self.blocked_servers,
                ]
                .iter()
                .all(|list| list.is_empty()))
    }

    /// Evaluates this configuration against an invite from `sender`, per the processing order of
    /// [MSC4155].
    ///
    /// A `default_action` of `block` takes precedence over the rule lists, and `enabled` only
    /// deactivates the rule lists, so [MSC4155] filtering never relaxes [invite blocking].
    ///
    /// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
    /// [invite blocking]: https://spec.matrix.org/v1.19/client-server-api/#invite-permission
    #[cfg(feature = "unstable-msc4155")]
    pub fn permission(&self, sender: &UserId) -> InvitePermission {
        if self.default_action == Some(InvitePermissionAction::Block) {
            return InvitePermission::Block;
        }

        if !self.enabled {
            return InvitePermission::Allow;
        }

        let user = sender.as_str();
        let server = sender.server_name().host();

        if matches(&self.allowed_users, user) {
            return InvitePermission::Allow;
        }
        if matches(&self.ignored_users, user) {
            return InvitePermission::Ignore;
        }
        if matches(&self.blocked_users, user) {
            return InvitePermission::Block;
        }
        if matches(&self.allowed_servers, server) {
            return InvitePermission::Allow;
        }
        if matches(&self.ignored_servers, server) {
            return InvitePermission::Ignore;
        }
        if matches(&self.blocked_servers, server) {
            return InvitePermission::Block;
        }

        InvitePermission::Allow
    }
}

/// `enabled` defaults to `true`, so the derive only serves the build without
/// [MSC4155]'s rule lists.
///
/// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
#[cfg(feature = "unstable-msc4155")]
impl Default for InvitePermissionConfigEventContent {
    fn default() -> Self {
        Self {
            default_action: None,
            enabled: true,
            allowed_users: Vec::new(),
            ignored_users: Vec::new(),
            blocked_users: Vec::new(),
            allowed_servers: Vec::new(),
            ignored_servers: Vec::new(),
            blocked_servers: Vec::new(),
        }
    }
}

/// The result of evaluating an invite against an [`InvitePermissionConfigEventContent`].
///
/// The semantics of `ignore` and `block` are defined in [MSC4283]: ignoring hides the invite with
/// no feedback to the inviter, whereas blocking rejects it with `M_INVITE_BLOCKED`.
///
/// [MSC4283]: https://github.com/matrix-org/matrix-spec-proposals/pull/4283
#[cfg(feature = "unstable-msc4155")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum InvitePermission {
    /// The invite proceeds normally.
    Allow,

    /// The invite is accepted but never surfaced to the invitee via sync or push.
    Ignore,

    /// The invite is rejected.
    Block,
}

/// Whether any glob in `globs` matches `value`, case-insensitively like [server ACLs].
///
/// [server ACLs]: https://spec.matrix.org/v1.19/client-server-api/#mroomserver_acl
#[cfg(feature = "unstable-msc4155")]
fn matches(globs: &[String], value: &str) -> bool {
    globs.iter().any(|glob| match glob.contains(['*', '?']) {
        true => WildMatch::new_case_insensitive(glob).matches(value),
        false => lowercase(glob).eq(lowercase(value)),
    })
}

/// The case folding [`WildMatch`] applies, for a pattern holding no wildcard.
#[cfg(feature = "unstable-msc4155")]
fn lowercase(value: &str) -> impl Iterator<Item = char> + '_ {
    value.chars().flat_map(char::to_lowercase)
}

/// Possible actions in response to an invite.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum InvitePermissionAction {
    /// Reject the invite.
    Block,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The content of an [`org.matrix.msc4380.invite_permission_config`][MSC4380] account data, the
/// unstable version of [`InvitePermissionConfigEventContent`].
///
/// Controls whether invites to this account are permitted.
///
/// [MSC4380]: https://github.com/matrix-org/matrix-spec-proposals/pull/4380
#[cfg(feature = "unstable-msc4380")]
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    kind = GlobalAccountData,
    type = "org.matrix.msc4380.invite_permission_config",
)]
pub struct UnstableInvitePermissionConfigEventContent {
    /// When set to true, indicates that the user does not wish to receive *any* room invites, and
    /// they should be blocked.
    #[serde(default, deserialize_with = "ruma_common::serde::default_on_error")]
    pub block_all: bool,
}

#[cfg(feature = "unstable-msc4380")]
impl UnstableInvitePermissionConfigEventContent {
    /// Creates a new `UnstableInvitePermissionConfigEventContent` from the desired boolean state.
    pub fn new(block_all: bool) -> Self {
        Self { block_all }
    }
}

#[cfg(feature = "unstable-msc4380")]
impl From<UnstableInvitePermissionConfigEventContent> for InvitePermissionConfigEventContent {
    fn from(value: UnstableInvitePermissionConfigEventContent) -> Self {
        Self {
            default_action: value.block_all.then_some(InvitePermissionAction::Block),
            ..Default::default()
        }
    }
}

#[cfg(feature = "unstable-msc4380")]
impl From<InvitePermissionConfigEventContent> for UnstableInvitePermissionConfigEventContent {
    fn from(value: InvitePermissionConfigEventContent) -> Self {
        Self {
            block_all: value
                .default_action
                .is_some_and(|action| matches!(action, InvitePermissionAction::Block)),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    #[cfg(feature = "unstable-msc4380")]
    use super::UnstableInvitePermissionConfigEventContent;
    use super::{InvitePermissionAction, InvitePermissionConfigEventContent};
    use crate::AnyGlobalAccountDataEvent;

    #[cfg(feature = "unstable-msc4380")]
    #[test]
    fn unstable_serialization() {
        let invite_permission_config = UnstableInvitePermissionConfigEventContent::new(true);

        assert_to_canonical_json_eq!(
            invite_permission_config,
            json!({
                "block_all": true,
            }),
        );
    }

    #[cfg(feature = "unstable-msc4380")]
    #[test]
    fn unstable_deserialization() {
        let json = json!({
            "content": {
                "block_all": true,
            },
            "type": "org.matrix.msc4380.invite_permission_config",
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::UnstableInvitePermissionConfig(ev))
        );
        assert!(ev.content.block_all);
    }

    #[test]
    fn stable_serialization() {
        let mut invite_permission_config = InvitePermissionConfigEventContent::new();
        assert_to_canonical_json_eq!(invite_permission_config, json!({}),);

        invite_permission_config.default_action = Some(InvitePermissionAction::Block);
        assert_to_canonical_json_eq!(
            invite_permission_config,
            json!({
                "default_action": "block",
            }),
        );
    }

    #[test]
    fn stable_deserialization() {
        let json = json!({
            "content": {
                "default_action": "block",
            },
            "type": "m.invite_permission_config",
        });
        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::InvitePermissionConfig(ev))
        );
        assert_eq!(ev.content.default_action, Some(InvitePermissionAction::Block));

        let json = json!({
            "content": {},
            "type": "m.invite_permission_config",
        });
        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::InvitePermissionConfig(ev))
        );
        assert_eq!(ev.content.default_action, None);
    }

    #[cfg(feature = "unstable-msc4155")]
    mod msc4155 {
        use ruma_common::user_id;
        use serde_json::{from_value as from_json_value, json};

        use super::super::{InvitePermission, InvitePermissionConfigEventContent};

        fn config(value: serde_json::Value) -> InvitePermissionConfigEventContent {
            from_json_value(value).unwrap()
        }

        #[test]
        fn empty_config_allows() {
            let config = config(json!({}));
            assert!(config.enabled);
            assert_eq!(config.permission(user_id!("@alice:example.org")), InvitePermission::Allow);
        }

        #[test]
        fn processing_order() {
            let config = config(json!({
                "allowed_users": ["@goodguy:badguys.org"],
                "ignored_users": ["@meh:badguys.org"],
                "blocked_users": ["@notactuallyguy:goodguys.org"],
                "allowed_servers": ["goodguys.org"],
                "ignored_servers": ["reallybadguys.org"],
                "blocked_servers": ["*"],
            }));

            assert_eq!(
                config.permission(user_id!("@goodguy:badguys.org")),
                InvitePermission::Allow
            );
            assert_eq!(config.permission(user_id!("@meh:badguys.org")), InvitePermission::Ignore);
            assert_eq!(
                config.permission(user_id!("@notactuallyguy:goodguys.org")),
                InvitePermission::Block
            );
            assert_eq!(
                config.permission(user_id!("@anyone:goodguys.org")),
                InvitePermission::Allow
            );
            assert_eq!(
                config.permission(user_id!("@rando:reallybadguys.org")),
                InvitePermission::Ignore
            );
            assert_eq!(
                config.permission(user_id!("@rando:elsewhere.org")),
                InvitePermission::Block
            );
        }

        #[test]
        fn user_rules_precede_server_rules() {
            let config = config(json!({
                "blocked_users": ["@badguy:goodguys.org"],
                "allowed_servers": ["goodguys.org"],
            }));

            assert_eq!(
                config.permission(user_id!("@badguy:goodguys.org")),
                InvitePermission::Block
            );
            assert_eq!(
                config.permission(user_id!("@goodguy:goodguys.org")),
                InvitePermission::Allow
            );
        }

        #[test]
        fn inert_configs() {
            assert!(config(json!({})).is_inert());
            assert!(config(json!({"allowed_users": []})).is_inert());
            assert!(config(json!({"enabled": false, "blocked_servers": ["*"]})).is_inert());

            assert!(!config(json!({"blocked_servers": ["*"]})).is_inert());
            assert!(!config(json!({"default_action": "block"})).is_inert());
            assert!(!config(json!({"default_action": "block", "enabled": false})).is_inert());
        }

        #[test]
        fn disabled_config_allows() {
            let config = config(json!({
                "enabled": false,
                "blocked_servers": ["*"],
            }));

            assert_eq!(config.permission(user_id!("@alice:example.org")), InvitePermission::Allow);
        }

        #[test]
        fn default_action_block_precedes_rules() {
            let allowlisted = config(json!({
                "default_action": "block",
                "allowed_users": ["@alice:example.org"],
            }));

            assert_eq!(
                allowlisted.permission(user_id!("@alice:example.org")),
                InvitePermission::Block
            );

            let disabled = config(json!({
                "default_action": "block",
                "enabled": false,
            }));

            assert_eq!(
                disabled.permission(user_id!("@alice:example.org")),
                InvitePermission::Block
            );
        }

        #[test]
        fn server_globs_ignore_ports_and_case() {
            let config = config(json!({
                "blocked_servers": ["BadGuys.org"],
            }));

            assert_eq!(
                config.permission(user_id!("@rando:badguys.org:8448")),
                InvitePermission::Block
            );
        }

        #[test]
        fn glob_wildcards_match_users() {
            let config = config(json!({
                "ignored_users": ["@spam*:*"],
            }));

            assert_eq!(
                config.permission(user_id!("@spammer123:anywhere.example")),
                InvitePermission::Ignore
            );
            assert_eq!(
                config.permission(user_id!("@ham:anywhere.example")),
                InvitePermission::Allow
            );
        }

        #[test]
        fn invalid_list_invalidates_event() {
            assert!(
                from_json_value::<InvitePermissionConfigEventContent>(json!({
                    "blocked_users": "not-an-array",
                }))
                .is_err()
            );

            assert!(
                from_json_value::<InvitePermissionConfigEventContent>(json!({
                    "blocked_users": [42],
                }))
                .is_err()
            );
        }
    }
}
