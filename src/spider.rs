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
    pub async fn click_treeitem(&self, treeitem: &WebElement) -> Result<()> {
        let treeitem_header = treeitem
            .find(By::ClassName("node__header__labelWrapper___dJ7OH"))
            .await
            .context("Could not find treeitem header!")?;
        treeitem_header
            .click()
            .await
            .context("Treeitem not clickable!")?;

        Ok(())
    }

    pub async fn click_treeitem_toggle(&self, treeitem: WebElement) -> Result<()> {
        let treeitem_toggle = treeitem
            .find(By::ClassName(
                "node__header__chevron___zXVME reset__reset___2e25U",
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
