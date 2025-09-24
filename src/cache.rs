use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Default, Serialize, Deserialize)]
pub struct BuildCache {
    pub templates_hash: String,
    pub pages: HashMap<String, String>,
}

pub fn load(out_root: &Path) -> BuildCache {
    let p = out_root.join(".ssg-cache.json");
    match fs::read(&p) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => BuildCache::default(),
    }
}

pub fn save(out_root: &Path, cache: &BuildCache) -> io::Result<()> {
    let p = out_root.join(".ssg-cache.json");
    let bytes = serde_json::to_vec_pretty(cache).unwrap();
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(p, bytes)
}

pub fn file_hash(path: &Path) -> io::Result<String> {
    let data = fs::read(path)?;
    Ok(blake3::hash(&data).to_hex().to_string())
}

pub fn templates_hash(dir: &Path) -> io::Result<String> {
    let mut files: Vec<PathBuf> = Vec::new();
    for e in WalkDir::new(dir) {
        let e = match e {
            Ok(x) => x,
            Err(_) => continue,
        };
        if e.file_type().is_file() {
            files.push(e.into_path());
        }
    }
    files.sort();

    let mut hasher = blake3::Hasher::new();
    for p in files {
        hasher.update(p.to_string_lossy().as_bytes());
        if let Ok(data) = fs::read(&p) {
            hasher.update(&data);
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, thread, time::Duration};
    use tempfile::tempdir;

    #[test]
    fn file_hash_changes_when_file_changes() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("a.txt");
        fs::write(&f, "hello").unwrap();
        let h1 = file_hash(&f).unwrap();

        // ensure mtime can change on coarse filesystems
        thread::sleep(Duration::from_millis(10));
        fs::write(&f, "hello!").unwrap();
        let h2 = file_hash(&f).unwrap();

        assert_ne!(h1, h2);
    }

    #[test]
    fn templates_hash_changes_on_update() {
        let dir = tempdir().unwrap();
        let tpls = dir.path().join("templates");
        fs::create_dir(&tpls).unwrap();
        let base = tpls.join("base.html");
        fs::write(&base, "<title>{{ page.title }}</title>").unwrap();

        let h1 = templates_hash(&tpls).unwrap();

        thread::sleep(Duration::from_millis(10));
        fs::write(&base, "<title>{{ page.title }} X</title>").unwrap();
        let h2 = templates_hash(&tpls).unwrap();

        assert_ne!(h1, h2);
    }
}
