use once_cell::sync::Lazy;
use std::fs;
use std::time::Duration;
use thirtyfour::{prelude::*, ElementPredicate};

static URL: &str = "https://cms.schrackforstudents.com/neos/login";
static USERNAME: &str = "mkronberger";
static PASSWORD: Lazy<String> = Lazy::new(|| {
    fs::read_to_string("/run/secrets/cms-pswd").unwrap_or_else(|_| "your_fallback_here".to_string())
    // Fallbackstring add your password here if you dont use agenix
});

async fn login(driver: &WebDriver) -> WebDriverResult<()> {
    let username_field = driver.find(By::Id("username")).await?;

    let password_field = driver.find(By::Id("password")).await?;

    let login_button = driver.find(By::ClassName("neos-login-btn")).await?;

    driver
        .action_chain()
        .click_element(&username_field)
        .send_keys(USERNAME)
        .click_element(&password_field)
        .send_keys(PASSWORD.to_string())
        .click_element(&login_button)
        .perform()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::firefox();

    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    driver.get(URL).await?;

    login(&driver).await?;

    println!("Press Enter to close the browser...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    driver.quit().await?;

    Ok(())
}
