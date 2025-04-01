// src/lib.rs

pub mod error;
pub mod tree;

pub use error::MyLibraryError;
pub use tree::{init_logger, FileNode, FileTree};
