//! # Janus Plugins
//!
//! This crate provides a set of plugins for the Janus WebRTC Gateway.
//!
//! Currently, it supports:
//! - EchoTest plugin
//! - AudioBridge plugin
//! - VideoRoom plugin
//! - Streaming plugin (minimal support)
//!
//! All of the plugins are hidden behind feature flags to allow you to cherry-pick your dependencies. By default, all plugins are enabled.
//!
//! If you can't find an API you're looking for, it might be hidden behind the `__experimental` feature since it's
//! not well tested yet. Alternatively, you could construct the body and send it, as every plugin handler dereferences to [`JaHandle`](jarust_core::jahandle::JaHandle).
//!

#[macro_use]
mod from;

#[macro_use]
mod make_dto;

#[cfg(feature = "echo-test")]
pub mod echo_test;

#[cfg(feature = "audio-bridge")]
pub mod audio_bridge;

#[cfg(feature = "video-room")]
pub mod video_room;

#[cfg(feature = "streaming")]
pub mod streaming;

#[cfg(feature = "legacy-video-room")]
pub mod legacy_video_room;

pub mod common;
pub use common::JanusId;
