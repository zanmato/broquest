//! Core domain types for the Broquest API client
//!
//! This module contains the fundamental data types that are shared across
//! multiple modules without creating circular dependencies.

mod http;
mod request;

pub use http::*;
pub use request::*;
