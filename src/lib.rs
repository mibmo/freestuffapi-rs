#![warn(missing_docs)]

//! Client for the freestuffbot.xyz API.
//!
//! Main interface is through the [`Client`] struct, which is constructed using
//! [`Client::builder()`].
//!
//! An API key can be gotten from <https://freestuffbot.xyz>.
//!
//! # Usage
//! ```no_run
//! # use freestuffapi::Client;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let api_key = "secret api key";
//! let client = Client::builder()
//!     .key(&api_key)
//!     .build()?;
//! client.ping().await?;
//! #     Ok(())
//! # }
//! ```

pub mod api;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub use client::Client;
