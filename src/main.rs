mod tree;

use crossterm::event::{read, Event, KeyCode};
use csv::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::time::Duration;
use thirtyfour::{prelude::*, support};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tree::FileTree;

static URL: &str = "https://cms.schrackforstudents.com/neos/login";
static TAGPATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tags.csv");

#[derive(serde::Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

pub async fn login(driver: &WebDriver) -> WebDriverResult<()> {
    // Read the secret file
    let secret_content = match fs::read_to_string("/run/secrets/cms-pswd") {
        Ok(content) => content,
        Err(_) => {
            println!("Notice: No secret file found. Please type in credentials manually.");
            return Ok(()); // Stop execution as we can't proceed without credentials
        }
    };

    let credentials: Credentials = match serde_json::from_str(&secret_content) {
        Ok(credentials) => credentials,
        Err(e) => {
            eprintln!("Failed to parse secret file: {}", e);
            return Ok(());
        }
    };

    let username = credentials.username;
    let password = credentials.password;

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

    // right now this takes the ready state of the login site, this should be for the cms site!

    loop {
        let ready_state: String = driver
            .execute("return document.readyState", Vec::new())
            .await?
            .json()
            .as_str()
            .unwrap_or("")
            .to_string();

        if ready_state == "complete" {
            break;
        }

        support::sleep(Duration::from_millis(500)).await;
    }

    println!("\nSite is loaded, you can now start manipulating the site.");

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
    let filetree = driver
        .find(By::Css(".style__pageTree___1vfOV > div:nth-child(1)"))
        .await?;
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
            println!("Tree item id: { }\n", text);
            file.write_all(text.as_bytes()).await?;
            file.write_all(b"\n").await?
        } else {
            println!("Treeitem has no id");
        }
        if let Some(text) = item.attr("title").await? {
            println!("Tree item: { }\n", text);
            file.write_all(text.as_bytes()).await?;
            file.write_all(b"\n").await?
        } else {
            println!("Treeitem has no title attribute");
        }
    }

    Ok(())
}

async fn expand_all_collapsed(driver: &WebDriver) -> WebDriverResult<()> {
    let tree_container = driver.find(By::Css(".style__pageTree___1vfOV")).await?;

    loop {
        let collapsed_buttons = tree_container
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

async fn select_by_id(driver: &WebDriver, locator: &By) -> WebDriverResult<Option<WebElement>> {
    match driver.find(locator.clone()).await {
        Ok(el) => Ok(Some(el)),
        Err(_) => Ok(None),
    }
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::firefox();

    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    driver.get(URL).await?;

    login(&driver).await?;

    let welcome_message = r#"
    Welcome to the tag spider you can do the following actions by pressing the given keys

    q -> quit the program
    a -> add tags (must be in question answer environment)
    c -> clear tags (must be in question answer environment)
    e -> expand all the tree items present
    t -> test a function which is defined in main.rs
    "#;

    loop {
        // welcome message is printed twice sometimes?
        println!("{}", welcome_message);

        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('a') => add_tags(false, &driver).await?,
                KeyCode::Char('c') => add_tags(true, &driver).await?,
                KeyCode::Char('e') => {
                    // use with caution! This takes pretty long, but has to be tested.
                    expand_all_collapsed(&driver).await?;
                    list_tree(&driver).await?;
                }
                KeyCode::Char('t') => {
                    expand_all_collapsed(&driver).await?;

                    // Build the tree.
                    let file_tree = FileTree::build_tree(&driver).await?;

                    let json_str = serde_json::to_string_pretty(&file_tree)
                        .expect("Failed to serialize file tree to JSON");

                    tokio::fs::write("tree.json", &json_str)
                        .await
                        .expect("Failed to write tree.json");
                }
                _ => {}
            }
        }
    }

    driver.quit().await?;

    Ok(())
}
