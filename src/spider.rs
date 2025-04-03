// src/spider.rs
use crate::error::SpiderError;
use crate::FileTree;
use log::info;
use std::path::PathBuf;
use thirtyfour::{prelude::*, WebDriver};

pub struct Spider {
    pub driver: WebDriver,
    pub root: FileTree,
}

impl Spider {
    pub async fn new<C>(capabilities: C, url: &str, tree_path: PathBuf) -> Result<Self, SpiderError>
    where
        C: Into<Capabilities>,
    {
        let driver = WebDriver::new("http://localhost:4444", capabilities).await?;
        driver.get(url).await?;
        let root = FileTree::from_json_file(tree_path)?;
        Ok(Self { driver, root })
    }

    pub async fn toggle_treeitem(&self, selector: &str) -> WebDriverResult<()> {
        println!("Test");
        let element = self.driver.find(By::Css(selector)).await?;
        let expanded = element.attr("aria-expanded").await?;
        let value = match expanded.as_deref() {
            Some("true") => "false",
            Some("false") => "true",
            _ => "true",
        };
        info! {"Now we try to run JS"}
        let js = format!(
            r#"
            const el = document.querySelector("{selector}");
            if (el) {{
                el.setAttribute("aria-expanded", "{value}");
                return true;
            }} else {{
                return false;
            }}
            "#,
            selector = selector,
            value = value,
        );

        self.driver.execute(&js, Vec::new()).await?;
        Ok(())
    }
}
