use include_dir::*;
use sha2::Digest;
use std::path::*;
use std::collections::*;

static ASSETS_DIR: Dir<'_> = include_dir!("./assets");

fn recursive_assets(dir: &'static Dir) -> HashMap<String, Vec<u8>> {
    let mut result = HashMap::new();

    for file in dir.files() {
        let path = String::from(file.path().to_str().unwrap());
        let value = file.contents().to_vec();

        result.insert(path, value);
    }

    for relative in dir.dirs() {
        result.extend(recursive_assets(relative));
    }

    result
}

/// Load embedded build assets
pub fn load_assets() -> HashMap<String, Vec<u8>> {
    recursive_assets(&ASSETS_DIR)
}

fn recursive_paths(path: PathBuf) -> HashSet<PathBuf> {
    let mut result = HashSet::new();
    let dir = std::fs::read_dir(path).expect("Dir read error");

    for entry in dir {
        let entry = entry.unwrap();

        let path = entry.path();
        if path.is_dir() {
            result.extend(recursive_paths(path));
            continue;
        }

        result.insert(path);
    }

    result
}

pub fn assets_paths(path: impl Into<PathBuf>) -> HashSet<PathBuf> {
    recursive_paths(path.into())
}

pub fn digest(value: &Vec<u8>) -> Vec<u8> {
    sha2::Sha256::digest(value).into_iter().collect()
}