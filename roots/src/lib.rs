//! Transactional append-only B-Tree storage for `BonsaiDb`.

#![forbid(unsafe_code)]
#![warn(
    clippy::cargo,
    missing_docs,
    // clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    future_incompatible,
    rust_2018_idioms,
)]
#![cfg_attr(doc, deny(rustdoc::all))]
#![allow(
    clippy::missing_errors_doc, // TODO clippy::missing_errors_doc
    clippy::option_if_let_else,
    clippy::module_name_repetitions,
)]

mod async_file;
mod error;
mod roots;
mod transaction;
mod tree;
mod vault;

#[cfg(test)]
mod test_util;

#[cfg(feature = "uring")]
pub use self::async_file::uring::UringFile;
pub use self::{
    async_file::{tokio::TokioFile, AsyncFile, File},
    error::Error,
    roots::Roots,
    vault::Vault,
};
