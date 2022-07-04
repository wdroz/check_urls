pub mod checkurls;
pub mod common;

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};

use checkurls::{get_files, BadUrls};
use clap::Parser;
use common::Message;
use flume::{self, Sender};
use reqwest::{self};

/// Verify the validity of URLs inside your files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path of codebase to check
    #[clap(short, long, default_value = ".")]
    path: String,
}

#[tokio::main]
async fn main() -> Result<(), i32> {
    let args = Args::parse();
    let folder = args.path;
    let (tx, rx) = flume::unbounded();
    let (tx_url, rx_url) = flume::unbounded();
    let visited_url = Arc::new(Mutex::new(HashSet::new()));
    let has_bad_urls = Arc::new(Mutex::new(false));
    tokio::spawn(async move {
        while let Ok(message) = rx.recv() {
            check_urls(message, tx_url.clone()).await;
            //check_urls(message, &tx_url).await;
        }
    });
    get_files(folder, tx, &visited_url).await;
    let has_bad_urls_clone = has_bad_urls.clone();
    let _ = tokio::spawn(async move {
        while let Ok(bad_url) = rx_url.recv() {
            println!("{bad_url}");
            {
                let mut has_bad_urls_value = has_bad_urls_clone.lock().unwrap();
                *has_bad_urls_value = true;
            }
        }
    })
    .await;
    let final_has_bad_urls = has_bad_urls.lock().unwrap();
    if *final_has_bad_urls {
        println!("⚠️ some files contains invalid URLs ⚠️");
        Err(1)
    } else {
        Ok(())
    }
}

/// Check that the URLs are responding (quickly enough)
///
/// # Arguments
///
/// * `message` - The message containing the url and path
/// * `tx_url` - The channel to report fault URLs
async fn check_urls(message: Message, tx_url: Sender<BadUrls>) {
    let url = &message.url;
    let path = message.path;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let resp = client.get(url).send().await;
    match resp {
        Ok(good_response) => {
            if !good_response.status().is_success() {
                let badurl = BadUrls {
                    from: path,
                    url: url.clone(),
                    info: good_response.status().to_string(),
                };
                let _ = tx_url.send(badurl);
            }
        }
        Err(error) => {
            let badurl = BadUrls {
                from: path,
                url: url.clone(),
                info: error.to_string(),
            };
            let _ = tx_url.send(badurl);
        }
    }
}
