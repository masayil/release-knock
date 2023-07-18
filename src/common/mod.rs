pub mod http;
use anyhow::anyhow;
use std::path::PathBuf;

pub fn is_config_exist(s: &str) -> anyhow::Result<PathBuf> {
    let mut path = PathBuf::new();
    path.push(s);
    if path.is_file() {
        Ok(path)
    } else {
        Err(anyhow!(
            "\"{}\" is not a regular file or not exist",
            path.display()
        ))
    }
}
