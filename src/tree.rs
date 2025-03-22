#![allow(unused)]

use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thirtyfour::{error::WebDriverResult, By, WebDriver};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub id: String,
    pub parent: Option<String>,
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

    /// Builds the entire file tree starting from the known root selector.
    /// In this example, we start at ".style__pageTree___1vfOV".
    pub async fn build_tree(driver: &WebDriver) -> WebDriverResult<FileTree> {
        let mut tree = FileTree::new();
        let root_selector = ".style__pageTree___1vfOV";
        // Start DFS traversal from the root; no parent for the root.
        Self::traverse_all_nodes(root_selector, driver, &mut tree, None).await?;
        Ok(tree)
    }

    /// Recursively traverses all possible child nodes of `selector` using a consecutive misses heuristic.
    #[async_recursion]
    async fn traverse_all_nodes(
        selector: &str,
        driver: &WebDriver,
        tree: &mut FileTree,
        parent_id: Option<String>,
    ) -> WebDriverResult<()> {
        let max_consecutive_misses = 3; // break after 3 consecutive misses
        let mut index = 1;
        let mut consecutive_misses = 0;

        while consecutive_misses < max_consecutive_misses {
            let current_selector = format!("{} > div:nth-child({})", selector, index);
            println!("Trying selector: {}", current_selector);

            match driver.find(By::Css(&current_selector)).await {
                Ok(element) => {
                    // Reset the miss counter because we found an element.
                    consecutive_misses = 0;
                    // Check if the element has the "aria-labelledby" attribute.
                    if let Ok(Some(attr)) = element.attr("aria-labelledby").await {
                        println!(
                            "Found node with aria-labelledby: {} at {}",
                            attr, current_selector
                        );
                        let node = FileNode {
                            id: attr.clone(),
                            parent: parent_id.clone(),
                            children: HashSet::new(),
                        };
                        tree.nodes.insert(attr.clone(), node);
                        if let Some(ref pid) = parent_id {
                            if let Some(parent_node) = tree.nodes.get_mut(pid) {
                                parent_node.children.insert(attr.clone());
                            }
                        }
                        // Recursively traverse from this node.
                        Self::traverse_all_nodes(&current_selector, driver, tree, Some(attr))
                            .await?;
                    } else {
                        println!(
                            "Element at {} does not have aria-labelledby. Descending further...",
                            current_selector
                        );
                        // Even if the element itself is not valid, try descending further.
                        Self::traverse_all_nodes(
                            &current_selector,
                            driver,
                            tree,
                            parent_id.clone(),
                        )
                        .await?;
                    }
                }
                Err(_) => {
                    println!("No element found at {}.", current_selector);
                    consecutive_misses += 1;
                }
            }
            index += 1;
        }
        println!(
            "Breaking out of traversal at selector {} after {} consecutive misses.",
            selector, consecutive_misses
        );
        Ok(())
    }
}
