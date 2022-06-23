use regex::Regex;
use std::fs;
use std::{future::Future, path::Path};
// Or maybe use Stream?
use futures::executor::block_on;
use futures::{
    stream::{FuturesUnordered, StreamExt},
    FutureExt,
};
use reqwest;

use glob::{glob, Pattern, PatternError};

async fn read_urls_from_file(file: String) -> Option<Vec<String>> {
    let regex = Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"#).unwrap();
    let contents = fs::read_to_string(file.clone());
    match contents {
        Ok(lines) => {
            let mut urls = Vec::new();
            for caps in regex.captures_iter(&lines) {
                for cap in caps.iter() {
                    if let Some(capture) = cap {
                        let url = capture.as_str().to_string();
                        if !url.is_empty() {
                            println!("pushing {}", url);
                            urls.push(url);
                        }
                    }
                }
            }
            Some(urls)
        }
        Err(_) => None,
    }
}

async fn check_url(url: String) -> bool {
    let client = reqwest::Client::new();
    let res = client.head(url).send().await;
    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn print_result(r: bool) {
    println!("result is {}", r);
}

async fn handle_urls(maybe_urls: Option<Vec<String>>) {
    if let Some(urls) = maybe_urls {
        let mut workers = FuturesUnordered::new();
        for url in urls {
            workers.push(check_url(url))
        }
        workers.for_each_concurrent(20, print_result).await;
    }
}

async fn process_files(files: Vec<String>) {
    let mut workers = FuturesUnordered::new();
    for file in files {
        workers.push(read_urls_from_file(file.clone()));
    }
    workers.for_each_concurrent(20, handle_urls).await;
}

fn get_files(glob_expr: &str) -> Result<Option<Vec<String>>, PatternError> {
    let glob_res = glob(glob_expr);
    let mut res = Vec::new();
    match glob_res {
        Ok(paths) => {
            for entry in paths {
                if let Ok(path) = entry {
                    if path.is_file() {
                        res.push(String::from(path.to_string_lossy()));
                    }
                }
            }
            Ok(Some(res))
        }
        Err(e) => Err(e),
    }
}

fn main() {
    let maybe_files = get_files("*.md").expect("error with the pattern");
    if let Some(files) = maybe_files {
        block_on(process_files(files));
    }
}
