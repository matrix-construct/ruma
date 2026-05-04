//! Endpoints for client configuration.

#[cfg(feature = "unstable-msc3391")]
pub mod delete_global_account_data;
#[cfg(feature = "unstable-msc3391")]
pub mod delete_room_account_data;
pub mod get_global_account_data;
pub mod get_room_account_data;
pub mod set_global_account_data;
pub mod set_room_account_data;
