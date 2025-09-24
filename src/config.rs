use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SiteConfig {
    pub title: String,
    pub base_url: String,
    pub theme: String,
    pub description: Option<String>,
    pub author: Option<String>,

    #[serde(rename = "src")]
    pub src_dir: PathBuf,
    #[serde(rename = "out")]
    pub out_dir: PathBuf,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Site Title".to_string(),
            base_url: "http://localhost/".to_string(),
            theme: "default".to_string(),
            description: None,
            author: None,
            src_dir: PathBuf::from("src"),
            out_dir: PathBuf::from("out"),
        }
    }
}

pub fn load_config<P: AsRef<Path>>(root: P) -> io::Result<SiteConfig> {
    let root = root.as_ref();
    let path = root.join("site.toml");

    if !path.exists() {
        return Ok(SiteConfig::default());
    }

    let text = fs::read_to_string(&path)?;

    let mut config: SiteConfig =
        toml::from_str(&text).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    if config.src_dir.is_relative() {
        config.src_dir = root.join(&config.src_dir);
    }

    if config.out_dir.is_relative() {
        config.out_dir = root.join(&config.out_dir);
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let cfg = load_config(dir.path()).unwrap();
        assert_eq!(cfg.title, "Site Title");
        assert!(cfg.src_dir.ends_with("src"));
        assert!(cfg.out_dir.ends_with("out"));
    }

    #[test]
    fn parses_and_layers_defaults() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("site.toml");
        fs::write(
            &p,
            r#"
                title = "My Blog"
                base_url = "https://example.com/"
                src = "content"
                # theme omitted → default "default"
            "#,
        )
        .unwrap();

        let cfg = load_config(dir.path()).unwrap();
        assert_eq!(cfg.title, "My Blog");
        assert_eq!(cfg.theme, "default");
        assert!(cfg.src_dir.ends_with("content"));
        assert!(cfg.out_dir.ends_with("out"));
    }

    #[test]
    fn invalid_toml_is_invalid_data() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("site.toml"), "not = { valid = toml").unwrap();
        let err = load_config(dir.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
