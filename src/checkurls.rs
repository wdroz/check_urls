use crate::common::Message;
use flume::Sender;
use regex::Regex;

use std::{
    collections::HashSet,
    fmt,
    path::Path,
    sync::{Arc, Mutex},
};
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

pub async fn get_files(
    path: String,
    tx: Sender<Message>,
    visited_url: &Arc<Mutex<HashSet<String>>>,
) {
    for result in Walk::new(path) {
        match result {
            Ok(entry) => process_entry(&entry, &tx, visited_url).await,
            Err(err) => println!("ERROR: {}", err),
        }
    }
}

async fn process_entry(
    entry: &DirEntry,
    tx: &Sender<Message>,
    visited_url: &Arc<Mutex<HashSet<String>>>,
) {
    if let Some(file_type) = entry.file_type() {
        if !file_type.is_dir() {
            extract_urls(entry.path(), &tx, &visited_url).await;
        }
    }
}

async fn extract_urls(
    path: &Path,
    tx: &Sender<Message>,
    visited_url: &Arc<Mutex<HashSet<String>>>,
) {
    let re = Regex::new(r"((https?)://)(www.)?[a-z0-9]+\.[a-z]+(/[a-zA-Z0-9#]+/?)*").unwrap();
    if let Ok(mut file) = File::open(path).await {
        let mut contents = vec![];
        if let Ok(_) = file.read_to_end(&mut contents).await {
            for caps in re.captures_iter(std::str::from_utf8(&contents).unwrap()) {
                let url = caps.get(0).unwrap().as_str();
                {
                    let visited_url_lock = &mut *visited_url.lock().unwrap();
                    if !visited_url_lock.contains(url) {
                        visited_url_lock.insert(url.to_string());
                        _ = tx.send(Message {
                            path: path.to_string_lossy().to_string(),
                            url: url.to_string(),
                        });
                    }
                }
            }
        }
    }
}
