use crate::error::MyLibraryError;
use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
use thirtyfour::{prelude::ElementQueryable, By, WebDriver, WebElement};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub id: String,
    pub parent: Option<String>,
    #[serde(default)]
    pub children: HashSet<String>,
}

impl FileNode {
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FileTree {
    pub nodes: HashMap<String, FileNode>,
}

impl FileTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub async fn build_tree(driver: &WebDriver) -> Result<FileTree, MyLibraryError> {
        let mut tree = FileTree::new();

        // 1. Find the top-level tree container and all items
        let container = driver.find(By::Css("[role='tree']")).await?;
        let items = container.find_all(By::Css("[role='treeitem']")).await?;

        // 2. Insert each item into the tree, capturing its parent
        for item in &items {
            // The aria-labelledby attribute typically stores a unique ID or label
            if let Some(id) = item.attr("aria-labelledby").await? {
                let parent_id = find_parent_id(item).await?;

                // Insert node
                let node = FileNode {
                    id: id.clone(),
                    parent: parent_id.clone(),
                    children: HashSet::new(),
                };
                tree.nodes.insert(id, node);
            }
        }

        // 3. Link children to their parents
        for (id, node) in tree.nodes.clone() {
            if let Some(pid) = &node.parent {
                if let Some(parent_node) = tree.nodes.get_mut(pid) {
                    parent_node.children.insert(id);
                }
            }
        }

        // 4. Validate and return
        tree.validate()?;
        Ok(tree)
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
        }

        if root_count == 0 {
            return Err(MyLibraryError::Custom(
                "No root node found in the tree".to_string(),
            ));
        }

        // If you only expect exactly 1 root, keep the old check:
        // if root_count != 1 { ... }

        Ok(())
    }

    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, MyLibraryError> {
        let data = fs::read_to_string(path)?;
        let tree: FileTree = serde_json::from_str(&data)
            .map_err(|e| MyLibraryError::Custom(format!("JSON parse error: {}", e)))?;
        tree.validate()?;
        Ok(tree)
    }

    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), MyLibraryError> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// Recursively climbs up the DOM to find a parent tree item.
/// Returns the `aria-labelledby` ID of the parent if found.
#[async_recursion]
async fn find_parent_id(item: &WebElement) -> Result<Option<String>, MyLibraryError> {
    // Go one level up
    let parent = match item.query(By::XPath("..")).first().await {
        Ok(el) => el,
        Err(_) => return Ok(None), // No parent => root
    };

    // If the parent is itself a treeitem, we found our parent
    if let Some(role) = parent.attr("role").await? {
        if role == "treeitem" {
            // Return the parent's aria-labelledby as the parent ID
            return Ok(parent.attr("aria-labelledby").await?);
        }
    }

    // Otherwise, keep climbing up
    find_parent_id(&parent).await
}

/// Initialize the logger (call from main or lib entry point).
pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}
