//! Endpoints for application service encryption keys.

#[cfg(feature = "unstable-msc3983")]
pub mod claim_keys;
#[cfg(feature = "unstable-msc3984")]
pub mod query_keys;
