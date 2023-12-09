//! This library makes it easy to handle errors with [thiserror](https://crates.io/crates/thiserror).
//!
//! ```toml
//! [dependencies]
//! tosserror = "0.1"
//! ```
//!
//!
//! ## Example
//!
//! ```ignore
//! use thiserror::Error;
//! use tosserror::Toss;
//!
//! #[derive(Error, Toss, Debug)]
//! pub enum DataStoreError {
//!     #[error("invalid value ({value}) encountered")]
//!     InvalidValue {
//!         value: i32,
//!         source: std::num::TryFromIntError,
//!     },
//!     #[error("data store disconnected with msg {msg}: {status}")]
//!     Disconnect{
//!         status: u8,
//!         msg: String,
//!         source: std::io::Error
//!     }
//! }
//!
//! // uses
//! get_value().toss_invalid_value(123)?;
//!
//! // lazily provide context
//! data_store_fn().toss_disconnect_with(|| (123, "some msg".to_owned()))?;
//! ```
//!
//! See [Toss](derive.Toss.html) for available attributes.

pub use tosserror_derive::*;

#[cfg(feature = "thiserror")]
pub use thiserror;
#[cfg(feature = "thiserror")]
pub use thiserror::Error;
