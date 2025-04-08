// src/main.rs
use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode};
use csv::Reader;
use tag_spider_rs::filenode::FileNode;
use tag_spider_rs::tree::FileTree;
use std::path::PathBuf;
use std::{collections::HashMap, fs, time::Duration};
use tag_spider_rs::spider::Spider;
use thirtyfour::{prelude::*, support, By, WebDriver};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

static URL: &str = "https://cms.schrackforstudents.com/neos/login";
static TAGPATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tags.csv");

#[derive(serde::Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

/// Log in using the provided WebDriver.
pub async fn login(driver: &WebDriver) -> Result<()> {
    let secret_filepath = PathBuf::from("/run/secrets/cms-pswd");

    let secret_content = match fs::read_to_string(&secret_filepath) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "Warning: Could not read secret file in: {:?}, Error: {}",
                secret_filepath, e
            );
            return Ok(());
        }
    };

    let credentials: Credentials = serde_json::from_str(&secret_content)
        .context("Secrets are not valid Json with fields 'password' and 'username'")?;

    // Find the login elements
    let username_field = driver
        .find(By::Id("username"))
        .await
        .context("Could not find username field!")?;

    let password_field = driver
        .find(By::Id("password"))
        .await
        .context("Could not find a password field!")?;

    let login_button = driver
        .find(By::ClassName("neos-login-btn"))
        .await
        .context("Could not find login button!")?;

    // Perform the login action
    driver
        .action_chain()
        .click_element(&username_field)
        .send_keys(&credentials.username)
        .click_element(&password_field)
        .send_keys(&credentials.password)
        .click_element(&login_button)
        .perform()
        .await?;

    support::sleep(Duration::from_secs(2)).await;

    Ok(())
}

/// Load CSV data for tags.
fn load_csv_data(path: &str) -> Result<HashMap<String, String>> {
    let mut tags: HashMap<String, String> = HashMap::new();
    let mut reader = Reader::from_path(path)?;

    for line in reader.records() {
        let record = line?;
        tags.insert(record[0].to_string(), record[1].to_string());
    }

    Ok(tags)
}

/// Example function to add tags.
async fn add_tags(clear: bool, driver: &WebDriver) -> Result<()> {
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
            .key_down(thirtyfour::Key::Control)
            .send_keys("a")
            .key_up(thirtyfour::Key::Control)
            .send_keys(thirtyfour::Key::Backspace)
            .perform()
            .await?;

        if !clear {
            if let Some(val) = tags.get(id) {
                tag_textbox.send_keys(val).await?;
            } else {
                eprintln!("Error: key {} not found! Skipping...", id);
                iframe.clone().enter_frame().await?;
                continue;
            }
        }

        let apply_button = driver
            .query(By::Css("#neos-Inspector-Apply"))
            .first()
            .await?;
        apply_button.click().await?;

        println!("{} -> {}", id, value);
        iframe.clone().enter_frame().await?;
        support::sleep(Duration::new(1, 0)).await;
    }
    driver.enter_default_frame().await?;
    Ok(())
}

/// List tree items and save them to a file.
async fn list_tree(driver: &WebDriver) -> Result<()> {
    let filetree = driver.find(By::Css(".div:nth-child(1)")).await?;
    let tree_items = filetree
        .find_all(By::Css(
            ".style__pageTree___1vfOV > div:nth-child(1) [role='treeitem']",
        ))
        .await?;

    let path = "filetree.txt";
    let mut file = File::create(path).await?;

    println!("Number of tree items: {}", tree_items.len());
    for item in tree_items {
        if let Some(text) = item.id().await? {
            println!("Tree item id: {}\n", text);
            file.write_all(text.as_bytes()).await?;
            file.write_all(b"\n").await?;
        } else {
            println!("Treeitem has no id");
        }
        if let Some(text) = item.attr("title").await? {
            println!("Tree item: {}\n", text);
            file.write_all(text.as_bytes()).await?;
            file.write_all(b"\n").await?;
        } else {
            println!("Treeitem has no title attribute");
        }
    }
    Ok(())
}

/// Expand all collapsed items in the tree.
async fn expand_all_collapsed(driver: &WebDriver) -> Result<()> {
    let tree = driver.find(By::Css("[role='tree']")).await?;

    loop {
        let collapsed_buttons = tree
            .find_all(By::Css("a[class*='node__header__chevron--isCollapsed']"))
            .await?;
        if collapsed_buttons.is_empty() {
            println!("No more collapsed items found.");
            break;
        }
        println!("Found {} collapsed items.", collapsed_buttons.len());
        for button in collapsed_buttons {
            button.scroll_into_view().await?;
            button.click().await?;
            support::sleep(Duration::from_millis(500)).await;
        }
        support::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let filetree = FileTree::from_json_file(PathBuf::from("../resources/tree.json"));
    let filenode = FileNode::new_root(String::from("someid"), children);
    // Start the WebDriver session.
    let spider = Spider::new(
        DesiredCapabilities::firefox(),
        URL,
        ,
    )
    .await?;

    // Log in.
    login(&spider.driver).await?;

    let welcome_message = r#"
    Welcome to the tag spider. You can do the following actions by pressing:

    q -> quit the program
    a -> add tags (must be in question answer environment)
    c -> clear tags (must be in question answer environment)
    e -> expand all the tree items present
    t -> test tree building (builds, saves, and validates the file tree)
    p -> test opening and closing treeitems
    "#;

    loop {
        println!("{}", welcome_message);
        if let Event::Key(event) = crossterm::event::read().unwrap() {
            match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('a') => add_tags(false, &spider.driver).await?,
                KeyCode::Char('c') => add_tags(true, &spider.driver).await?,
                KeyCode::Char('e') => {
                    expand_all_collapsed(&spider.driver).await?;
                    list_tree(&spider.driver).await?;
                }
                KeyCode::Char('t') => {}
                KeyCode::Char('p') => {
                    let selector = "div[aria-labelledby='treeitem-2b58d239-label']";
                    spider.toggle_treeitem(selector).await?;
                }
                _ => {}
            }
        }
    }

    spider.driver.quit().await?;
    Ok(())
}
