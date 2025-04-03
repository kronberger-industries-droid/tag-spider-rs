// src/lib.rs

pub mod error;
pub mod spider;
pub mod tree;

pub use error::{init_logger, SpiderError};
pub use spider::Spider;
pub use tree::{FileNode, FileTree};
