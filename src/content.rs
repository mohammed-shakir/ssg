use std::path::{Path, PathBuf};
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
}
