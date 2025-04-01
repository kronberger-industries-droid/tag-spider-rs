// src/lib.rs

pub mod error;
pub mod tree;
pub mod ui;

pub use error::MyLibraryError;
pub use tree::{init_logger, FileNode, FileTree};
pub use ui::click_item;
