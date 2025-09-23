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
