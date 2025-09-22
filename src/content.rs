use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn collect_markdown_files<P: AsRef<Path>>(root: P) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(root) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Walk error: {}", err);
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        #[allow(clippy::collapsible_if)]
        if let Some(extension) = entry.path().extension().and_then(|ext| ext.to_str()) {
            if extension.eq_ignore_ascii_case("md") {
                result.push(entry.into_path());
            }
        }
    }

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontMatterFormat {
    Yaml,
    Toml,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageMeta {
    pub title: Option<String>,
    pub date: Option<String>,
    pub draft: bool,
    pub tags: Vec<String>,
    pub template: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document<M> {
    pub path: PathBuf,
    pub front_matter: Option<M>,
    pub body: String,
}

fn split_front_matter(text: &str) -> io::Result<(Option<(FrontMatterFormat, String)>, String)> {
    let mut text = text.to_owned();

    if text.starts_with('\u{FEFF}') {
        text = text.trim_start_matches('\u{FEFF}').to_string();
    }
    if text.contains('\r') {
        text = text.replace("\r\n", "\n").replace('\r', "\n");
    }

    // check foor yaml front matter
    if let Some(rest) = text.strip_prefix("---\n") {
        let Some(end) = rest
            .find("\n---\n")
            .or_else(|| rest.strip_suffix("\n---").map(|s| s.len()))
        else {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unclosed YAML front matter (---)",
            ));
        };
        let (fm, body) = rest.split_at(end);
        // Skip the closing fence line (account for presence/absence of trailing \n)
        let body = body
            .strip_prefix("\n---\n")
            .or_else(|| body.strip_prefix("\n---"))
            .unwrap_or(body);
        return Ok((
            Some((FrontMatterFormat::Yaml, fm.to_string())),
            body.to_string(),
        ));
    } else if let Some(rest) = text.strip_prefix("+++\n") {
        // check for toml front matter
        let Some(end) = rest
            .find("\n+++\n")
            .or_else(|| rest.strip_suffix("\n+++").map(|s| s.len()))
        else {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unclosed TOML front matter (+++)",
            ));
        };
        let (fm, body) = rest.split_at(end);
        let body = body
            .strip_prefix("\n+++\n")
            .or_else(|| body.strip_prefix("\n+++"))
            .unwrap_or(body);
        return Ok((
            Some((FrontMatterFormat::Toml, fm.to_string())),
            body.to_string(),
        ));
    }

    Ok((None, text))
}

pub fn load_document<M: DeserializeOwned>(path: impl AsRef<Path>) -> io::Result<Document<M>> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;

    let (front_matter, body) = split_front_matter(&content)?;

    let fm = match front_matter {
        Some((FrontMatterFormat::Yaml, fm_str)) => {
            let meta: M = serde_yaml::from_str(&fm_str).map_err(|e| {
                io::Error::new(ErrorKind::InvalidData, format!("YAML front matter: {e}"))
            })?;
            Some(meta)
        }
        Some((FrontMatterFormat::Toml, fm_str)) => {
            let meta: M = toml::from_str(&fm_str).map_err(|e| {
                io::Error::new(ErrorKind::InvalidData, format!("TOML front matter: {e}"))
            })?;
            Some(meta)
        }
        None => None,
    };

    Ok(Document {
        path: path.to_path_buf(),
        front_matter: fm,
        body: body.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io};
    use tempfile::tempdir;

    #[test]
    fn collects_only_md_files() -> io::Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        fs::create_dir_all(root.join("posts/sub"))?;

        fs::write(root.join("README.MD"), b"# readme")?;
        fs::write(root.join("notes.txt"), b"not markdown")?;
        fs::write(root.join("posts/a.md"), b"# a")?;
        fs::write(root.join("posts/sub/b.Md"), b"# b")?;

        let mut got = collect_markdown_files(root);
        got.sort(); // make comparison stable

        let mut expected = vec![
            root.join("README.MD"),
            root.join("posts/a.md"),
            root.join("posts/sub/b.Md"),
        ];
        expected.sort();

        assert_eq!(got, expected);
        Ok(())
    }

    #[test]
    fn returns_empty_when_no_markdown_files() -> io::Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        fs::create_dir_all(root.join("empty/sub"))?;

        let files = collect_markdown_files(root);
        assert!(files.is_empty());
        Ok(())
    }

    #[test]
    fn doc_with_valid_yaml_front_matter() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("post.md");
        fs::write(
            &f,
            "\
---\n\
title: Hello\n\
tags: [rust, ssg]\n\
draft: false\n\
---\n\
# Heading\n\
Body here.\n",
        )
        .unwrap();

        let doc = load_document::<PageMeta>(&f).unwrap();
        let meta = doc.front_matter.unwrap();
        assert_eq!(meta.title.as_deref(), Some("Hello"));
        assert_eq!(meta.tags, vec!["rust", "ssg"]);
        assert_eq!(doc.body.lines().next().unwrap(), "# Heading");
    }

    #[test]
    fn doc_with_no_front_matter() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("nofm.md");
        fs::write(&f, "# No FM\nPlain text").unwrap();

        let doc = load_document::<PageMeta>(&f).unwrap();
        assert!(doc.front_matter.is_none());
        assert!(doc.body.starts_with("# No FM"));
    }

    #[test]
    fn invalid_front_matter_errors() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("bad.md");
        // Broken YAML (missing colon)
        fs::write(&f, "---\n title Hello\n---\nbody").unwrap();

        let err = load_document::<PageMeta>(&f).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
