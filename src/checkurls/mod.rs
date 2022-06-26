use gitignore::File;
use glob::glob;
use std::path::Path;

pub fn get_files(path: String, ignore_file: String) {
    println!("{} and {}", path, ignore_file);
    let path_to_gitignore = Path::new(&ignore_file);
    let maybe_file = gitignore::File::new(&path_to_gitignore);
    if let Ok(file) = maybe_file {
        println!(".gitignore file OK");
        let mut glob_expression = path;
        print!("{}", glob_expression);
        if glob_expression.ends_with("/") {
            glob_expression.push_str("**/*");
        } else {
            glob_expression.push_str("/**/*");
        }
        println!("glob expr {}", glob_expression.to_string());
        for my_file in glob(&glob_expression).unwrap().into_iter() {
            if let Ok(good_file) = my_file {
                if file.is_excluded(&good_file).unwrap() {
                    println!("skipping");
                } else {
                    println!("{}", good_file.display());
                }
            }
        }
    } else {
        println!("issue with gitignore file");
    }
}
