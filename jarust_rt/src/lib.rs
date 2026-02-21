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
use std::time::Duration;

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

/// Sleeps for the given duration. Uses `tokio::time::sleep` on native and
/// `gloo_timers` on WASM (which does not support `std::time::Instant`).
#[cfg(not(target_family = "wasm"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// Sleeps for the given duration. Uses `tokio::time::sleep` on native and
/// `gloo_timers` on WASM (which does not support `std::time::Instant`).
#[cfg(target_family = "wasm")]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}

/// Runs `future` with a deadline. Uses `tokio::time::timeout` on native and
/// a `gloo_timers`-based race on WASM (which does not support `std::time::Instant`).
#[cfg(not(target_family = "wasm"))]
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, ()>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future).await.map_err(|_| ())
}

/// Runs `future` with a deadline. Uses `tokio::time::timeout` on native and
/// a `gloo_timers`-based race on WASM (which does not support `std::time::Instant`).
#[cfg(target_family = "wasm")]
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, ()>
where
    F: Future<Output = T>,
{
    use futures_util::future::Either;
    use futures_util::pin_mut;

    let timer = gloo_timers::future::sleep(duration);
    pin_mut!(future);
    pin_mut!(timer);

    match futures_util::future::select(future, timer).await {
        Either::Left((output, _)) => Ok(output),
        Either::Right((_, _)) => Err(()),
    }
}
