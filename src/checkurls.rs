use crate::common::Message;
use flume::Sender;
use regex::Regex;

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

pub async fn get_files(path: String, tx: Sender<Message>) {
    for result in Walk::new(path) {
        match result {
            Ok(entry) => process_entry(&entry, &tx).await,
            Err(err) => println!("ERROR: {}", err),
        }
    }
}

async fn process_entry(entry: &DirEntry, tx: &Sender<Message>) {
    if let Some(file_type) = entry.file_type() {
        if !file_type.is_dir() {
            extract_urls(entry.path().as_ref(), &tx).await;
        }
    }
}

async fn extract_urls(path: &Path, tx: &Sender<Message>) {
    let re = Regex::new(r"((https?)://)(www.)?[a-z0-9]+\.[a-z]+(/[a-zA-Z0-9#]+/?)*").unwrap();
    if let Ok(mut file) = File::open(path).await {
        let mut contents = vec![];
        if let Ok(_) = file.read_to_end(&mut contents).await {
            for caps in re.captures_iter(std::str::from_utf8(&contents).unwrap()) {
                let url = caps.get(0).unwrap().as_str();
                _ = tx.send(Message {
                    path: path.to_string_lossy().to_string(),
                    url: url.to_string(),
                });
            }
        }
    }
}
