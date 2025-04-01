use crate::error::MyLibraryError;
use std::time::Duration;
use thirtyfour::{prelude::ElementQueryable, By, WebDriver};

pub async fn click_item(driver: &WebDriver, id: String) -> Result<(), MyLibraryError> {
    // Find the element by its id.
    let item = driver.query(By::Css(format!("#{}", id))).first().await?;

    // First, try clicking the element.
    if let Err(click_err) = item.click().await {
        // If clicking fails, check if the parent element is collapsed.
        if let Ok(parent) = item.query(By::XPath("..")).first().await {
            // Look for an attribute that indicates expansion, e.g. "aria-expanded".
            if let Some(expanded) = parent.attr("aria-expanded").await? {
                if expanded == "false" {
                    // The parent is collapsed, so click it to expand.
                    parent.click().await?;
                    // Wait a short moment for the parent to expand.
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    // Now try clicking the original item again.
                    item.click().await?;
                    return Ok(());
                }
            }
        }
        return Err(MyLibraryError::Custom(format!(
            "Failed to click item {}: {}",
            id, click_err
        )));
    }

    Ok(())
}
