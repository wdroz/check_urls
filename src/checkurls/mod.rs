use std::path::Path;
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
            println!("FILE {}", entry.path().display());
            check_urls(entry.path().as_ref()).await;
        }
    }
}

async fn check_urls(path: &Path) {
    if let Ok(mut file) = File::open(path).await {
        let mut contents = vec![];
        if let Ok(_) = file.read_to_end(&mut contents).await {
            println!("len = {}", contents.len());
        }
    }
}
