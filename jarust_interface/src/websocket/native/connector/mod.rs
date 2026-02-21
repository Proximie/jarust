#[cfg(all(feature = "use-rustls", feature = "use-native-tls"))]
compile_error!("Feature \"rustls\" and feature \"native-tls\" cannot be enabled at the same time");

#[cfg(not(any(feature = "use-rustls", feature = "use-native-tls")))]
compile_error!("Either feature \"rustls\" or \"native-tls\" must be enabled for this crate");

#[cfg(feature = "use-native-tls")]
#[path = "native_tls_adapter.rs"]
mod adapter;

#[cfg(feature = "use-rustls")]
#[path = "rustls_adapter.rs"]
mod adapter;

pub use adapter::connect_async;
