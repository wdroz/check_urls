use std::{future::Future, path::Path};
use std::fs;
use regex::Regex;

use glob::{glob, Pattern, PatternError};

async fn read_urls_from_file(file: String) -> Option<Vec<String>> {
    let regex = Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();
    let contents = fs::read_to_string(file.clone());
    match contents {
        Ok(lines) => {
            let mut urls = Vec::new();
            for caps in regex.captures_iter(&lines) {
                for cap in caps.iter() {
                    if let Some(capture) = cap {
                        let url = capture.as_str().to_string();
                        urls.push(url);
                    }
                }
            }
            Some(urls)
        }
        Err(_) => None,
    }    
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
    println!("Hello, world!");
    get_files("*.md");
}
