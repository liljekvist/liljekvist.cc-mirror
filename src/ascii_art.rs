use std::fs;
use std::path::Path;
use rand::prelude::IndexedRandom;

/// Load all `*.txt` files from `dir` and return their contents as owned strings.
/// Files are sorted by name so the order is deterministic.
pub fn load(dir: &Path) -> Vec<String> {
    let mut styles: Vec<String> = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("Cannot read ascii_art dir {:?}: {e}", dir))
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "txt")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let content = fs::read_to_string(entry.path())
            .unwrap_or_else(|e| panic!("Cannot read {:?}: {e}", entry.path()));
        styles.push(content);
    }

    if styles.is_empty() {
        panic!("No .txt files found in ascii_art directory {:?}", dir);
    }

    styles
}

/// Pick a random style from a pre-loaded collection.
pub fn random(styles: &[String]) -> &str {
    let mut rng = rand::rng();
    styles.choose(&mut rng).map(String::as_str).unwrap_or("")
}
