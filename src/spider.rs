use std::time::Duration;

use crate::tree::FileTree;
use anyhow::{bail, Context, Result};
use async_recursion::async_recursion;
use thirtyfour::{prelude::*, support, WebDriver};
use tokio::time::Instant;

pub struct Spider {
    pub driver: WebDriver,
    pub file_tree: FileTree,
}

impl Spider {
    pub async fn new<C>(capabilities: C, url: &str, file_tree: FileTree) -> Result<Self>
    where
        C: Into<Capabilities>,
    {
        let driver = WebDriver::new("http://localhost:4444", capabilities)
            .await
            .context("Failed to create WebDriver")?;
        driver.get(url).await.context("Failed to navigate to URL")?;

        Ok(Self { driver, file_tree })
    }

    #[async_recursion]
    pub async fn find_treeitem(&self, id: &str) -> Result<WebElement> {
        let current_node = self.file_tree.nodes.get(id).context(format!(
            "Could not find node with given id in the filetree! {}",
            id
        ))?;

        if let Some(parent_id) = &current_node.parent {
            // Check if already expanded before toggling
            let expanded = self
                .find_treeitem(parent_id)
                .await?
                .attr("aria-expanded")
                .await?;

            if expanded != Some("true".to_string()) {
                self.click_treeitem_toggle(parent_id).await?;
            }
        }

        // Now attempt to find the current node
        let selector = format!("div[aria-labelledby='{}']", id);

        match self.driver.find(By::Css(&selector)).await {
            Ok(treeitem) => {
                treeitem.scroll_into_view().await.expect("We found the treeitem already there is no reasont that we cant scroll to its location!");
                Ok(treeitem)
            }
            Err(_) => bail!("Something went wrong!"),
        }
    }

    pub async fn click_treeitem(&self, id: &str) -> Result<()> {
        let treeitem = self.find_treeitem(id).await?;
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

    pub async fn click_treeitem_toggle(&self, id: &str) -> Result<()> {
        let treeitem = self.find_treeitem(id).await?;
        let treeitem_toggle = treeitem
            .find(By::Css(
                "a.node__header__chevron___zXVME.reset__reset___2e25U",
            ))
            .await
            .context("Could not find toggle button in this element!")?;

        treeitem_toggle
            .click()
            .await
            .context("Could not click the toggle button!")?;

        Ok(())
    }

    async fn wait_content_load(&self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        loop {
            match self
                .driver
                .find(By::Css(".style__loadingIndicator__container___1yhsy"))
                .await
            {
                Ok(_) => {
                    support::sleep(Duration::from_millis(500)).await;
                }
                Err(_) => return Ok(()),
            }
            let loading_bars = self
                .driver
                .find_all(By::Css(".style__loadingIndicator__container___1yhsy"))
                .await?;
            if loading_bars.is_empty() {
                return Ok(());
            }
            if start.elapsed() > timeout {
                bail!("Loading bar did not disappear within the timeout period.");
            }
        }
    }

    pub async fn extract_content(&self, id: &str) -> Result<()> {
        self.click_treeitem(id).await?;

        support::sleep(Duration::from_secs(2)).await;

        self.driver
            .enter_frame(0)
            .await
            .context("Could not enter main content iFrame!")?;

        self.wait_content_load(Duration::from_secs(30)).await?;

        let content_collection = self
            .driver
            .find(By::Css(
                "html body.neos-backend div.container div.neos-contentcollection",
            ))
            .await
            .context("Could not find neos-contentcollection!")?;

        let elements = content_collection
            .query(By::Css(":scope > div"))
            .all_from_selector()
            .await?;

        for element in elements {
            element.scroll_into_view().await?;

            support::sleep(Duration::from_millis(500)).await;

            for field in element.find_all(By::Css("p, ul")).await? {
                println!("{}\n", field.text().await?);
            }
        }

        Ok(())
    }
}
