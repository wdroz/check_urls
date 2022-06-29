use regex::Regex;
use reqwest;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use ignore::{DirEntry, Walk};

pub async fn get_files(path: String) {
    for result in Walk::new(path) {
        match result {
            Ok(entry) => process_entry(&entry).await,
            Err(err) => println!("ERROR: {}", err),
        }
    }
}

async fn process_entry(entry: &DirEntry) {
    if let Some(file_type) = entry.file_type() {
        if !file_type.is_dir() {
            check_urls(entry.path().as_ref()).await;
        }
    }
}

async fn check_urls(path: &Path) {
    let re = Regex::new(r"((https?)://)(www.)?[a-z0-9]+\.[a-z]+(/[a-zA-Z0-9#]+/?)*").unwrap();
    if let Ok(mut file) = File::open(path).await {
        let mut contents = vec![];
        if let Ok(_) = file.read_to_end(&mut contents).await {
            for caps in re.captures_iter(std::str::from_utf8(&contents).unwrap()) {
                let url = caps.get(0).unwrap().as_str();
                // TODO remove this when centralize things
                if url.contains("github") {
                    continue;
                }
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .unwrap();
                let resp = client.get(url).send().await;
                match resp {
                    Ok(good_response) => {
                        if !good_response.status().is_success() {
                            println!(
                                "❌ {} - {}\n└──{:?}",
                                url,
                                good_response.status().to_string(),
                                path
                            );
                        }
                    }
                    Err(error) => {
                        println!("❌ {} - {}\n└──{:?}", url, error.to_string(), path)
                    }
                }
            }
        }
    }
}
