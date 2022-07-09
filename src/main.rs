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

/// get the number of dead links inside a folder or file
///
/// # Arguments
///
/// * `folder` - The folder or file to check
///
/// Returns the number of dead links
async fn get_number_of_dead_links(folder: &str) -> i32 {
    let (tx, rx) = flume::unbounded();
    let (tx_url, rx_url) = flume::unbounded();
    let visited_url = Arc::new(Mutex::new(HashSet::new()));
    let nb_bad_urls = Arc::new(Mutex::new(0));
    tokio::spawn(async move {
        while let Ok(message) = rx.recv() {
            let clone_a = tx_url.clone();
            tokio::spawn(async move {
                check_urls(message, clone_a.clone()).await;
            });
        }
    });
    get_files(folder.to_string(), tx, &visited_url).await;
    let nb_bad_urls = Arc::clone(&nb_bad_urls);
    let clone = Arc::clone(&nb_bad_urls);
    let _ = tokio::spawn(async move {
        while let Ok(bad_url) = rx_url.recv() {
            println!("{bad_url}");
            {
                let mut num = clone.lock().unwrap();
                *num += 1;
            }
        }
    })
    .await;
    // let final_nb_bad_urls = nb_bad_urls.lock().unwrap();
    let final_nb_bad_urls = nb_bad_urls.lock().unwrap();
    *final_nb_bad_urls
}

#[tokio::main(flavor = "multi_thread", worker_threads = 50)]
async fn main() -> Result<(), i32> {
    let args = Args::parse();
    let folder = args.path;
    if get_number_of_dead_links(&folder).await > 0 {
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
            let accepted_code = vec!["401", "403"];
            if !good_response.status().is_success()
                && !accepted_code.contains(&good_response.status().as_str())
            {
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

// tests

#[cfg(test)]
mod check_urls {

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_find_3_bad_urls() {
        let my_file = "tests/test_urls.md";
        let res = get_number_of_dead_links(my_file).await;
        assert!(res == 3);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_find_0_bad_urls() {
        let my_file = "tests/only_good.md";
        let res = get_number_of_dead_links(my_file).await;
        assert!(res == 0);
    }
}
