#![feature(specialization, negative_impls, const_fn, backtrace, bool_to_option)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate futures_core;
#[macro_use]
extern crate pin_project;

pub use futures_util::future::LocalBoxFuture;

/// Traits and types related to database backends.
pub mod backend;
/// Traits and types related to database connections.
pub mod connection;
/// Database errors.
pub mod error;
/// Backend syntax extensions.
pub mod extensions;
/// Traits and types related to database queries.
pub mod query;
/// SQL implementation of some traits in this library.
pub mod sql;
/// Traits and types for SQL types.
pub mod types;
pub(crate) mod utils;
/// Traits and types for database values.
pub mod values;
