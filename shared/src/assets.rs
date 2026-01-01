use include_dir::*;
use sha2::Digest;
use std::collections::*;

static ASSETS_DIR: Dir<'_> = include_dir!("./assets");

fn recursive_load(dir: &'static Dir) -> HashMap<String, Vec<u8>> {
    let mut result = HashMap::new();

    for file in dir.files() {
        let path = String::from(file.path().to_str().unwrap());
        let value = file.contents().to_vec();

        result.insert(path, value);
    }

    for relative in dir.dirs() {
        result.extend(recursive_load(relative));
    }

    result
}

pub fn load_assets() -> HashMap<String, Vec<u8>> {
    recursive_load(&ASSETS_DIR)
}

pub fn digest(value: &Vec<u8>) -> Vec<u8> {
    sha2::Sha256::digest(value).into_iter().collect()
}