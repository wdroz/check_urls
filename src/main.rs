use std::path::Path;

use glob::{glob, Pattern, PatternError};

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
