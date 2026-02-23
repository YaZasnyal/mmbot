//! Middleware implementations for wrapping plugins
//!
//! Middlewares are plugins that wrap other plugins to add cross-cutting functionality
//! like filtering, logging, rate limiting, etc.

pub mod ignore_self;

pub use ignore_self::IgnoreSelf;
