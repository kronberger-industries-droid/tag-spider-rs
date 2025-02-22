use crossterm::event::{read, Event, KeyCode};
use csv::Reader;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::io::Error;
use thirtyfour::prelude::*;

static URL: &str = "https://cms.schrackforstudents.com/neos/login";
static TAGPATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tags.csv");
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

fn load_csv_data(path: &str) -> Result<HashMap<String, String>, Error> {
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

async fn add_tags(clear: bool, driver: &WebDriver) -> WebDriverResult<()> {
    let tags = load_csv_data(TAGPATH)?;

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
    }

    driver.enter_default_frame().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::firefox();

    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    driver.get(URL).await?;

    login(&driver).await?;

    let books_collapse_selector = ".style__pageTree___1vfOV > div:nth-child(1) > div:nth-child(1) > div:nth-child(3) > div:nth-child(7) > div:nth-child(3) > div:nth-child(1) > div:nth-child(2) > div:nth-child(1) > a:nth-child(3) > svg:nth-child(1)".to_string();

    collapse_tree_item(&driver, &books_collapse_selector).await?;

    // let question_tree_parent = driver
    //     .query(By::Css(".style__pageTree___1vfOV > div:nth-child(1) > div:nth-child(1) > div:nth-child(3) > div:nth-child(7) > div:nth-child(3) > div:nth-child(2) > div:nth-child(2)"))
    //     .all_from_selector()
    //     .await?;

    // let parentname = question_tree_parent[0].text().await?;

    // println!("Parent name: {}", parentname);

    // let question_tree_children = question_tree_parent[0]
    //     .query(By::Css(".style__pageTree___1vfOV > div"))
    //     .all_from_selector()
    //     .await?;

    // println!("Found {} file tree entries:", question_tree_children.len());

    // for entry in question_tree_children.iter() {
    //     let text = entry.text().await?;
    //     println! {"File: {}", text};
    // }

    let welcome_message = r#"
    Welcome to the tag spider you can do the following actions by pressing the given keys

    q -> quit the program
    a -> add tags (must be in question answer environment) 
    c -> clear tags (must be in question answer environment)
    "#;

    println!("{}", welcome_message);

    loop {
        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('a') => add_tags(false, &driver).await?,
                KeyCode::Char('c') => add_tags(true, &driver).await?,
                _ => {}
            }
        }
    }

    driver.quit().await?;

    Ok(())
}
