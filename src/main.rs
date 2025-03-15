use crossterm::event::{read, Event, KeyCode, KeyEvent};
use csv::Reader;
use json::object::Iter;
use json::JsonValue;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::time::Duration;
use thirtyfour::prelude::*;

static URL: &str = "https://cms.schrackforstudents.com/neos/login";
static TAGPATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tags.csv");

pub async fn login(driver: &WebDriver) -> WebDriverResult<()> {
    // Read the secret file
    let secret_content = match fs::read_to_string("/run/secrets/cms-pswd") {
        Ok(content) => content,
        Err(_) => {
            println!("Notice: No secret file found. Please type in credentials manually.");
            return Ok(()); // Stop execution as we can't proceed without credentials
        }
    };

    // Parse the secret JSON
    let parsed_secret = json::parse(&secret_content).unwrap_or(JsonValue::Null);

    // Extract username
    let username = match parsed_secret["username"].as_str() {
        Some(name) => name.to_string(),
        None => {
            println!("Notice: No username found in the secret file.");
            return Ok(()); // Stop execution as we need a username
        }
    };

    // Extract password
    let password = match parsed_secret["password"].as_str() {
        Some(pass) => pass.to_string(),
        None => {
            println!("Notice: No password found in the secret file.");
            return Ok(()); // Stop execution as we need a password
        }
    };

    // Find the login elements
    let username_field = driver.find(By::Id("username")).await?;
    let password_field = driver.find(By::Id("password")).await?;
    let login_button = driver.find(By::ClassName("neos-login-btn")).await?;

    // Perform the login action
    driver
        .action_chain()
        .click_element(&username_field)
        .send_keys(&username)
        .click_element(&password_field)
        .send_keys(&password)
        .click_element(&login_button)
        .perform()
        .await?;

    println!("Login attempt completed.");

    Ok(())
}

fn load_csv_data(path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut tags: HashMap<String, String> = HashMap::new();
    let mut reader = Reader::from_path(path)?;

    for line in reader.records() {
        let record = line?;
        tags.insert(record[0].to_string(), record[1].to_string());
    }

    Ok(tags)
}

async fn collapse_tree_item(driver: &WebDriver, css_selector: &str) -> WebDriverResult<()> {
    let collapse_books = driver.query(By::Css(css_selector)).first().await?;

    collapse_books.click().await?;

    Ok(())
}

async fn extract_source(driver: &WebDriver) -> WebDriverResult<()> {
    let iframe = driver
        .query(By::Css(r#"iframe[name="neos-content-main"]"#))
        .first()
        .await?;

    iframe.clone().enter_frame().await?;

    let source = driver.source().await?;

    fs::write("page_source.html", source).expect("Unable to write file");
    Ok(())
}

async fn extract_content(driver: &WebDriver) -> WebDriverResult<()> {
    let iframe = driver
        .query(By::Css(r#"iframe[name="neos-content-main"]"#))
        .first()
        .await?;

    iframe.clone().enter_frame().await?;

    let parent = driver
        .query(By::Css(
            "html body.neos-backend div.container div.neos-contentcollection",
        ))
        .first()
        .await?
        .find_all(By::Css(
            "html body.neos-backend div.container div.neos-contentcollection div",
        ))
        .await?;

    let text = parent.first().expect("Nothing found").text().await?;
    println!("{}", text);
    Ok(())
}

async fn add_tags(clear: bool, driver: &WebDriver) -> WebDriverResult<()> {
    let tags = load_csv_data(TAGPATH).unwrap();

    let iframe = driver
        .query(By::Css(r#"iframe[name="neos-content-main"]"#))
        .first()
        .await?;

    iframe.clone().enter_frame().await?;

    let content_collection = driver
        .query(By::Css(
            "html body.neos-backend div.container div.neos-contentcollection",
        ))
        .first()
        .await?;

    let questions = content_collection
        .find_all(By::Css("p.neos-inline-editable.questionTitle"))
        .await?;

    for question in questions {
        question.scroll_into_view().await?;
        let text = question.text().await?;
        let id = text.split(' ').next().unwrap();
        let value = tags.get(id).unwrap();

        question.click().await?;

        driver.enter_default_frame().await?;

        let tag_textbox = driver
            .query(By::Css("#__neos__editor__property---Tags"))
            .first()
            .await?;

        driver
            .action_chain()
            .click_element(&tag_textbox)
            .key_down(Key::Control)
            .send_keys("a")
            .key_up(Key::Control)
            .send_keys(Key::Backspace)
            .perform()
            .await?;

        if !clear {
            match tags.get(id) {
                Some(_) => tag_textbox.send_keys(value).await?,
                None => {
                    eprintln!("Error: key {} not found! Skipping...", id);
                    iframe.clone().enter_frame().await?;
                    continue;
                }
            }
        }

        let apply_button = driver
            .query(By::Css("#neos-Inspector-Apply"))
            .first()
            .await?;

        apply_button.click().await?;

        println!("{} -> {}", id, value);

        iframe.clone().enter_frame().await?;
        thirtyfour::support::sleep(Duration::new(1, 0)).await;
    }

    driver.enter_default_frame().await?;

    Ok(())
}

async fn list_tree(driver: &WebDriver) -> WebDriverResult<()> {
    let filetree = driver.find(By::Css(".style__pageTree___1vfOV")).await?;

    let tree_items = filetree.find_all(By::Css("[role='treeitem']")).await?;

    println!("{length}", length = tree_items.len());
    for item in tree_items {
        let text = item.text().await?;
        println!("Tree item: {}", text);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::firefox();

    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    driver.get(URL).await?;

    login(&driver).await?;

    // let books_collapse_selector = ".style__pageTree___1vfOV > div:nth-child(1) > div:nth-child(1) > div:nth-child(3) > div:nth-child(7) > div:nth-child(3) > div:nth-child(1) > div:nth-child(2) > div:nth-child(1) > a:nth-child(3) > svg:nth-child(1)".to_string();

    // collapse_tree_item(&driver, &books_collapse_selector).await?;

    // let welcome_message = r#"
    // Welcome to the tag spider you can do the following actions by pressing the given keys

    // q -> quit the program
    // a -> add tags (must be in question answer environment)
    // c -> clear tags (must be in question answer environment)
    // e -> extract the full source (this is a test)
    // "#;

    // println!("{}", welcome_message);

    loop {
        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('a') => add_tags(false, &driver).await?,
                KeyCode::Char('c') => add_tags(true, &driver).await?,
                KeyCode::Char('e') => list_tree(&driver).await?,
                _ => {}
            }
        }
    }

    driver.quit().await?;

    Ok(())
}
