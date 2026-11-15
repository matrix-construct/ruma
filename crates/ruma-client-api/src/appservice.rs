//! Endpoints part of the application service extension of the client-server API

#[cfg(feature = "unstable-msc4512")]
pub mod fed_proxy;
pub mod request_ping;
pub mod set_room_visibility;
