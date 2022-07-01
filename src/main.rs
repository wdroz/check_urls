pub mod checkurls;
pub mod common;

use std::{fmt, time::Duration};

use checkurls::get_files;
use clap::Parser;
use common::Message;
use flume::{self};
use reqwest::{self};
/// Simple program to greet a person

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path of codebase to check
    #[clap(short, long, default_value = ".")]
    path: String,

    /// File that contains patterns     to ignore
    #[clap(short, long, default_value = ".gitignore")]
    ignore_file: String,
}

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

#[tokio::main]
async fn main() {
    // Calling `say_world()` does not execute the body of `say_world()`.
    let args = Args::parse();
    let folder = args.path;
    let (tx, rx) = flume::unbounded();
    tokio::spawn(async move {
        loop {
            let message = rx.recv().unwrap();
            check_urls(message).await;
        }
    });
    get_files(folder, tx).await;
}

async fn check_urls(message: Message) {
    let url = &message.url;
    let path = message.path;
    let mut bad_urls: Vec<BadUrls> = Vec::new();
    //let mut bad_urls: Mutex::new(bad_urls);
    //let bad_urls = Arc::new(bad_urls);
    if !url.contains("github") {
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
                    bad_urls.push(badurl);
                }
            }
            Err(error) => {
                let badurl = BadUrls {
                    from: path,
                    url: url.clone(),
                    info: error.to_string(),
                };

                bad_urls.push(badurl);
            }
        }
    }
}
