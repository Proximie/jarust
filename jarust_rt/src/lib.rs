//! # Jarust Runtime
//!
//! A runtime abstraction crate for jarust.
//!

#[cfg(not(target_family = "wasm"))]
#[cfg(not(any(feature = "tokio-rt")))]
compile_error!("Feature \"tokio-rt\" must be enabled for this crate.");

#[cfg(not(target_family = "wasm"))]
#[cfg(feature = "tokio-rt")]
#[path = "tokio_rt.rs"]
pub mod jatask;

#[cfg(target_family = "wasm")]
#[path = "wasm_rt.rs"]
pub mod jatask;

use futures_util::Future;
pub use jatask::JaTask;

/// Spawns a new task. The name field is for debugging purposes only.
#[cfg(not(target_family = "wasm"))]
#[tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(task_name = name))]
pub fn spawn<F>(name: &str, future: F) -> JaTask
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tracing::trace!("Spawning task");
    jatask::spawn(name, future)
}

/// Spawns a new task. The name field is for debugging purposes only.
#[cfg(target_family = "wasm")]
#[tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(task_name = name))]
pub fn spawn<F>(name: &str, future: F) -> JaTask
where
    F: Future + 'static,
{
    tracing::trace!("Spawning task");
    jatask::spawn(name, future)
}
