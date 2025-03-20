#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use crossterm::event::{read, Event, KeyCode};
use csv::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::time::Duration;
use thirtyfour::prelude::*;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
static URL: &str = "https://cms.schrackforstudents.com/neos/login";
static TAGPATH: &str = "/home/kronberger/Programming/rust/tag-spider-rs/resources/tags.csv";
struct Credentials {
    username: String,
    password: String,
}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Credentials {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "username" => _serde::__private::Ok(__Field::__field0),
                        "password" => _serde::__private::Ok(__Field::__field1),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"username" => _serde::__private::Ok(__Field::__field0),
                        b"password" => _serde::__private::Ok(__Field::__field1),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Credentials>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Credentials;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "struct Credentials",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<
                        String,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct Credentials with 2 elements",
                                ),
                            );
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<
                        String,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(
                                _serde::de::Error::invalid_length(
                                    1usize,
                                    &"struct Credentials with 2 elements",
                                ),
                            );
                        }
                    };
                    _serde::__private::Ok(Credentials {
                        username: __field0,
                        password: __field1,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "username",
                                        ),
                                    );
                                }
                                __field0 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                );
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "password",
                                        ),
                                    );
                                }
                                __field1 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => {
                            _serde::__private::de::missing_field("username")?
                        }
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => {
                            _serde::__private::de::missing_field("password")?
                        }
                    };
                    _serde::__private::Ok(Credentials {
                        username: __field0,
                        password: __field1,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["username", "password"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Credentials",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<Credentials>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
pub async fn login(driver: &WebDriver) -> WebDriverResult<()> {
    let secret_content = match fs::read_to_string("/run/secrets/cms-pswd") {
        Ok(content) => content,
        Err(_) => {
            {
                ::std::io::_print(
                    format_args!(
                        "Notice: No secret file found. Please type in credentials manually.\n",
                    ),
                );
            };
            return Ok(());
        }
    };
    let credentials: Credentials = match serde_json::from_str(&secret_content) {
        Ok(credentials) => credentials,
        Err(e) => {
            {
                ::std::io::_eprint(
                    format_args!("Failed to parse secret file: {0}\n", e),
                );
            };
            return Ok(());
        }
    };
    let username = credentials.username;
    let password = credentials.password;
    let username_field = driver.find(By::Id("username")).await?;
    let password_field = driver.find(By::Id("password")).await?;
    let login_button = driver.find(By::ClassName("neos-login-btn")).await?;
    driver
        .action_chain()
        .click_element(&username_field)
        .send_keys(&username)
        .click_element(&password_field)
        .send_keys(&password)
        .click_element(&login_button)
        .perform()
        .await?;
    {
        ::std::io::_print(format_args!("Login attempt completed.\n"));
    };
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
async fn collapse_tree_item(
    driver: &WebDriver,
    css_selector: &str,
) -> WebDriverResult<()> {
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
        .query(
            By::Css("html body.neos-backend div.container div.neos-contentcollection"),
        )
        .first()
        .await?
        .find_all(
            By::Css(
                "html body.neos-backend div.container div.neos-contentcollection div",
            ),
        )
        .await?;
    let text = parent.first().expect("Nothing found").text().await?;
    {
        ::std::io::_print(format_args!("{0}\n", text));
    };
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
        .query(
            By::Css("html body.neos-backend div.container div.neos-contentcollection"),
        )
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
                    {
                        ::std::io::_eprint(
                            format_args!("Error: key {0} not found! Skipping...\n", id),
                        );
                    };
                    iframe.clone().enter_frame().await?;
                    continue;
                }
            }
        }
        let apply_button = driver.query(By::Css("#neos-Inspector-Apply")).first().await?;
        apply_button.click().await?;
        {
            ::std::io::_print(format_args!("{0} -> {1}\n", id, value));
        };
        iframe.clone().enter_frame().await?;
        thirtyfour::support::sleep(Duration::new(1, 0)).await;
    }
    driver.enter_default_frame().await?;
    Ok(())
}
async fn list_tree(driver: &WebDriver) -> WebDriverResult<()> {
    let filetree = driver.find(By::Css(".style__pageTree___1vfOV")).await?;
    let tree_items = filetree.find_all(By::Css("[role='treeitem']")).await?;
    let path = "filetree.txt";
    let mut file = File::create(path).await?;
    {
        ::std::io::_print(format_args!("Number of tree items: {0}\n", tree_items.len()));
    };
    for item in tree_items {
        let text = item.text().await?;
        {
            ::std::io::_print(format_args!("Tree item: {0}\n\n", text));
        };
        file.write_all(text.as_bytes()).await?;
        file.write_all(b"\n").await?
    }
    Ok(())
}
fn main() -> WebDriverResult<()> {
    let body = async {
        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;
        driver.get(URL).await?;
        login(&driver).await?;
        let welcome_message = r#"
    Welcome to the tag spider you can do the following actions by pressing the given keys

    q -> quit the program
    a -> add tags (must be in question answer environment)
    c -> clear tags (must be in question answer environment)
    t -> run the current function to test (defined in source)
    "#;
        {
            ::std::io::_print(format_args!("{0}\n", welcome_message));
        };
        loop {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('a') => add_tags(false, &driver).await?,
                    KeyCode::Char('c') => add_tags(true, &driver).await?,
                    KeyCode::Char('t') => list_tree(&driver).await?,
                    _ => {}
                }
            }
        }
        driver.quit().await?;
        Ok(())
    };
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
