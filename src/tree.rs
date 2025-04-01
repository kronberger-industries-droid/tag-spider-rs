// src/tree.rs

use crate::error::MyLibraryError;
use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
use thirtyfour::{error::WebDriverError, By, WebDriver};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub id: String,
    pub parent: Option<String>,
    #[serde(default)]
    pub children: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileTree {
    pub nodes: HashMap<String, FileNode>,
}

impl FileTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Build the tree from a WebDriver, with improved error handling.
    pub async fn build_tree(driver: &WebDriver) -> Result<FileTree, MyLibraryError> {
        let mut tree = FileTree::new();
        let root_selector = "[role='tree']";
        Self::traverse_tree_nodes(driver, &mut tree, root_selector, None).await?;
        tree.validate()?;
        Ok(tree)
    }

    #[async_recursion]
    async fn traverse_tree_nodes(
        driver: &WebDriver,
        tree: &mut FileTree,
        selector: &str,
        parent_id: Option<&str>,
    ) -> Result<(), MyLibraryError> {
        let container = driver.find(By::Css(selector)).await?;
        let items = container.find_all(By::Css("[role='treeitem']")).await?;

        for item in items {
            if let Ok(Some(id)) = item.attr("aria-labelledby").await {
                let parent_id_cloned = parent_id.map(|s| s.to_string());

                // Add node to tree
                let node = FileNode {
                    id: id.clone(),
                    parent: parent_id_cloned.clone(),
                    children: HashSet::new(),
                };

                if tree.nodes.insert(id.clone(), node).is_some() {
                    return Err(MyLibraryError::Custom(format!("Duplicate node ID: {}", id)));
                }

                // Update parent's children
                if let Some(pid) = &parent_id_cloned {
                    tree.nodes
                        .get_mut(pid)
                        .ok_or_else(|| {
                            MyLibraryError::Custom(format!("Parent not found: {}", pid))
                        })?
                        .children
                        .insert(id.clone());
                }

                // Check for nested groups
                if let Ok(Some(expanded)) = item.attr("aria-expanded").await {
                    if expanded == "true" {
                        let sub_selector = format!("[aria-labelledby='{}'] [role='group']", id);
                        Self::traverse_tree_nodes(driver, tree, &sub_selector, Some(&id)).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate the consistency of the tree.
    pub fn validate(&self) -> Result<(), MyLibraryError> {
        let mut root_count = 0;

        for node in self.nodes.values() {
            if let Some(pid) = &node.parent {
                if !self.nodes.contains_key(pid) {
                    return Err(MyLibraryError::Custom(format!(
                        "Parent node {} not found for {}",
                        pid, node.id
                    )));
                }
            } else {
                root_count += 1;
            }

            for cid in &node.children {
                if !self.nodes.contains_key(cid) {
                    return Err(MyLibraryError::Custom(format!(
                        "Child node {} not found for {}",
                        cid, node.id
                    )));
                }
            }
        }

        if root_count != 1 {
            return Err(MyLibraryError::Custom(format!(
                "Expected exactly 1 root node, found {}",
                root_count
            )));
        }

        Ok(())
    }

    // ---- JSON Serialization ----

    /// Load a FileTree from a JSON file.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, MyLibraryError> {
        let data = fs::read_to_string(path)?;
        let tree: FileTree = serde_json::from_str(&data)
            .map_err(|e| MyLibraryError::Custom(format!("JSON parse error: {}", e)))?;
        tree.validate()?;
        Ok(tree)
    }

    /// Save the FileTree as a JSON file.
    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), MyLibraryError> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// Initialize the logger (call from main or lib entry point).
pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}
