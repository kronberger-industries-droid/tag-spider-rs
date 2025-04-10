use crate::filenode::FileNode;
use anyhow::{bail, Context, Result}; // Import anyhow
use thirtyfour::{prelude::*, WebDriver};

pub struct Spider {
    pub driver: WebDriver,
    pub current_node: FileNode,
}

impl Spider {
    pub async fn new<C>(capabilities: C, url: &str, root: &FileNode) -> Result<Self>
    where
        C: Into<Capabilities>,
    {
        let driver = WebDriver::new("http://localhost:4444", capabilities)
            .await
            .context("Failed to create WebDriver")?;
        driver.get(url).await.context("Failed to navigate to URL")?;

        let current_node = root.clone();

        Ok(Self {
            driver,
            current_node,
        })
    }

    pub async fn toggle_treeitem(&self, selector: &str) -> Result<()> {
        println!("Test");
        let element = self.driver.find(By::Css(selector)).await?;
        let expanded = element.attr("aria-expanded").await?;
        let value = match expanded.as_deref() {
            Some("true") => "false",
            Some("false") => "true",
            _ => bail!("Got none from aria-expanded"),
        };

        Ok(())
    }

    pub async fn click_treeitem_toggle(&self, treeitem: WebElement) -> Result<()> {
        let treeitem_toggle = treeitem
            .find(By::Css(
                "a[data-neos-integrational-test='tree_item_nodeHeader__subTreetoggle']",
            ))
            .await
            .context("Could not find toggle button in this element!")?;

        treeitem_toggle
            .click()
            .await
            .context("Could not click the toggle button!")?;

        Ok(())
    }
}
