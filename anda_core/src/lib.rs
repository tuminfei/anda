use std::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
};

pub mod agent;
pub mod context;
pub mod http;
pub mod tool;

pub use agent::*;
pub use context::*;
pub use http::*;
pub use tool::*;

/// A type alias for a boxed error that is thread-safe and sendable across threads.
/// This is commonly used as a return type for functions that can return various error types.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A type alias for a boxed future that is thread-safe and sendable across threads.
pub type BoxPinFut<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// A global state manager for Agent or Tool
///
/// Wraps any type `S` to provide shared state management with
/// automatic dereferencing capabilities
#[derive(Debug, Default, Clone, Copy)]
pub struct State<S>(pub S);

impl<S> Deref for State<S> {
    type Target = S;

    /// Provides immutable access to the inner state
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> DerefMut for State<S> {
    /// Provides mutable access to the inner state
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Joins two paths together
pub fn join_path(a: &Path, b: &Path) -> Path {
    Path::from(format!("{}/{}", a, b))
}
