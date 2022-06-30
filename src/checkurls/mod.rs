use regex::Regex;
use reqwest::{self};
use std::time::Duration;
use std::{fmt, path::Path};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use ignore::{DirEntry, Walk};
#[derive(Debug, Clone)]
struct BadUrls {
    url: String,
    from: String,
    info: String,
}

impl fmt::Display for BadUrls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "❌ {} - {}\n└──{:?}", self.url, self.from, self.info,)
    }
}

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
    let mut bad_urls: Vec<BadUrls> = Vec::new();
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
                            let badurl = BadUrls {
                                from: path.to_string_lossy().to_string(),
                                url: url.to_string(),
                                info: good_response.status().to_string(),
                            };
                            bad_urls.push(badurl);
                        }
                    }
                    Err(error) => {
                        let badurl = BadUrls {
                            from: path.to_string_lossy().to_string(),
                            url: url.to_string(),
                            info: error.to_string(),
                        };
                        bad_urls.push(badurl);
                    }
                }
            }
        }
    }
    for badurl in bad_urls {
        println!("{}", badurl);
    }
}
