//! Shared test helpers for Mattermost integration tests.
//!
//! Provides `MattermostTestEnv` for setting up isolated test environments
//! and `AuthenticatedClient` for making API calls.

mod client;
mod env;

pub use client::AuthenticatedClient;
pub use env::MattermostTestEnv;
