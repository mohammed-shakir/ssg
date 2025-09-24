use std::{
    fs, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::content::{Document, PageMeta};

pub fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut dash = false;
    for ch in s.chars().flat_map(|c| c.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            dash = false;
        } else if matches!(ch, ' ' | '-' | '_' | '.') && !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "untitled".into()
    } else {
        out
    }
}

pub fn out_path_for(
    src_root: &Path,
    out_root: &Path,
    md_path: &Path,
    doc: &Document<PageMeta>,
) -> PathBuf {
    let rel = md_path.strip_prefix(src_root).unwrap_or(md_path);

    if rel.file_stem().and_then(|s| s.to_str()) == Some("index") {
        let mut dest = out_root.join(rel);
        dest.set_extension("html");
        return dest;
    }

    let stem = rel
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");
    let fm_slug = doc
        .front_matter
        .as_ref()
        .and_then(|m| m.slug.as_deref())
        .unwrap_or(stem);
    let slug = slugify(fm_slug);

    let parent = rel.parent().unwrap_or_else(|| Path::new(""));
    out_root.join(parent).join(slug).join("index.html")
}

pub fn copy_static_assets(src_root: &Path, out_root: &Path) -> io::Result<()> {
    for entry in WalkDir::new(src_root) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("walk: {e}");
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }

        let p = entry.path();
        let rel = match p.strip_prefix(src_root) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if rel
            .components()
            .next()
            .is_some_and(|c| c.as_os_str() == "templates")
        {
            continue;
        }
        if rel.file_name().is_some_and(|n| n == "site.toml") {
            continue;
        }
        if p.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("md"))
        {
            continue;
        }

        let dest = out_root.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(p, &dest)?;
    }
    Ok(())
}

pub fn url_for_out_path(out_root: &Path, out_path: &Path) -> String {
    let rel: PathBuf = out_path
        .strip_prefix(out_root)
        .unwrap_or(out_path)
        .to_owned();
    if rel
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("index.html"))
    {
        let dir = rel.parent().unwrap_or(Path::new(""));
        let s = dir.to_string_lossy();
        if s.is_empty() {
            "/".to_string()
        } else {
            format!("/{}/", s.replace('\\', "/"))
        }
    } else {
        format!("/{}", rel.to_string_lossy().replace('\\', "/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Document, PageMeta};

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("My Post!"), "my-post");
        assert_eq!(slugify("   spaces   "), "spaces");
        assert_eq!(slugify(""), "untitled");
    }

    #[test]
    fn out_path_for_index_and_post() {
        let src = Path::new("/s");
        let out = Path::new("/o");

        let idx = Path::new("/s/index.md");
        let doc_index = Document::<PageMeta> {
            path: idx.into(),
            front_matter: None,
            body: String::new(),
        };
        let p = out_path_for(src, out, idx, &doc_index);
        assert_eq!(p, Path::new("/o/index.html"));

        let post = Path::new("/s/posts/first.md");
        let doc_post = Document::<PageMeta> {
            path: post.into(),
            front_matter: None,
            body: String::new(),
        };
        let p2 = out_path_for(src, out, post, &doc_post);
        assert_eq!(p2, Path::new("/o/posts/first/index.html"));

        let with_slug = Document::<PageMeta> {
            path: post.into(),
            front_matter: Some(PageMeta {
                slug: Some("custom-slug".into()),
                ..Default::default()
            }),
            body: String::new(),
        };
        let p3 = out_path_for(src, out, post, &with_slug);
        assert_eq!(p3, Path::new("/o/posts/custom-slug/index.html"));
    }

    #[test]
    fn url_for_out_path_index_rules() {
        assert_eq!(
            url_for_out_path(Path::new("/o"), Path::new("/o/index.html")),
            "/"
        );
        assert_eq!(
            url_for_out_path(Path::new("/o"), Path::new("/o/posts/first/index.html")),
            "/posts/first/"
        );
        assert_eq!(
            url_for_out_path(Path::new("/o"), Path::new("/o/raw.html")),
            "/raw.html"
        );
    }
}
