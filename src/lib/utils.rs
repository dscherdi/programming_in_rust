use std::{
    path::{PathBuf},
    fs
};

use crate::UserData;

pub fn read_dir(paths: &[PathBuf]) -> Vec<String> {
    let mut valid: Vec<String> = Vec::new();

    for path in paths {
        if path.exists() {
            match path.read_dir() {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let parent = entry.path().parent().unwrap().file_name().unwrap().to_str().to_owned().unwrap().to_string();
                            let file_name = entry.file_name().to_str().to_owned().unwrap().to_string();
                            valid.push(format!("{}/{}", parent, file_name));
                        }
                    }
                }
                Err(_) => {},
            }
        }
    }

    valid
}

pub fn fetch_users() -> Vec<UserData> {
	let contents = fs::read_to_string("users.json")
		.expect("Something went wrong reading the users file");
	serde_json::from_str(contents.as_str()).expect("Something went wrong parsing the json")
}