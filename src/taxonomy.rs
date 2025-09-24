use serde::Serialize;
use std::{collections::HashMap, fs, io, path::Path};

use crate::{
    config::SiteConfig,
    content::Document,
    routing::{slugify, url_for_out_path},
    templates::Templates,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PageSummary {
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
}

pub fn group_by_tag(pages: &[PageSummary]) -> HashMap<String, Vec<&PageSummary>> {
    use std::collections::hash_map::Entry;
    let mut map: HashMap<String, Vec<&PageSummary>> = HashMap::new();
    for p in pages {
        for tag in &p.tags {
            let key = tag.to_lowercase();
            match map.entry(key) {
                Entry::Occupied(mut e) => e.get_mut().push(p),
                Entry::Vacant(v) => {
                    v.insert(vec![p]);
                }
            }
        }
    }
    map
}

pub fn write_tag_pages(
    templates: &Templates,
    cfg: &SiteConfig,
    out_root: &Path,
    pages: &[PageSummary],
) -> io::Result<()> {
    let groups = group_by_tag(pages);

    #[derive(Serialize)]
    struct TagPage<'a> {
        name: &'a str,
        pages: Vec<&'a PageSummary>,
        url: String,
    }
    #[derive(Serialize)]
    struct TagsIndex<'a> {
        tags: Vec<TagSummary<'a>>,
    }
    #[derive(Serialize)]
    struct TagSummary<'a> {
        name: &'a str,
        count: usize,
        url: String,
    }

    for (tag_name, items) in groups.iter() {
        let slug = slugify(tag_name);
        let out_path = out_root.join("tags").join(&slug).join("index.html");
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tag_vm = TagPage {
            name: tag_name,
            pages: items.clone(),
            url: format!("/tags/{}/", slug),
        };
        let html = render_tag(templates, cfg, &tag_vm)?;
        fs::write(out_path, html)?;
    }

    let mut all: Vec<_> = groups
        .iter()
        .map(|(name, items)| TagSummary {
            name,
            count: items.len(),
            url: format!("/tags/{}/", slugify(name)),
        })
        .collect();
    all.sort_by(|a, b| a.name.cmp(b.name));

    let index_vm = TagsIndex { tags: all };
    let html = render_tags_index(templates, cfg, &index_vm)?;
    let out_path = out_root.join("tags").join("index.html");
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out_path, html)?;
    Ok(())
}

fn render_tag(templates: &Templates, cfg: &SiteConfig, tag: &impl Serialize) -> io::Result<String> {
    use tera::Context;
    let mut ctx = Context::new();
    ctx.insert("site", cfg);
    ctx.insert("tag", tag);
    templates.tera().render("tag.html", &ctx).map_err(to_io)
}

fn render_tags_index(
    templates: &Templates,
    cfg: &SiteConfig,
    tags: &impl Serialize,
) -> io::Result<String> {
    use tera::Context;
    let mut ctx = Context::new();
    ctx.insert("site", cfg);
    ctx.insert("tags", tags);
    templates.tera().render("tags.html", &ctx).map_err(to_io)
}

fn to_io(e: tera::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e)
}

pub fn summarize<M>(
    _doc: &Document<M>,
    out_root: &Path,
    out_path: &Path,
    tags: &[String],
    title: &str,
) -> PageSummary {
    PageSummary {
        title: title.to_string(),
        url: url_for_out_path(out_root, out_path),
        tags: tags.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_pages_by_tag() {
        let pages = vec![
            PageSummary {
                title: "A".into(),
                url: "/a/".into(),
                tags: vec!["rust".into(), "ssg".into()],
            },
            PageSummary {
                title: "B".into(),
                url: "/b/".into(),
                tags: vec!["rust".into()],
            },
            PageSummary {
                title: "C".into(),
                url: "/c/".into(),
                tags: vec!["cli".into()],
            },
        ];

        let g = group_by_tag(&pages);
        assert_eq!(g.get("rust").unwrap().len(), 2);
        assert_eq!(g.get("ssg").unwrap().len(), 1);
        assert_eq!(g.get("cli").unwrap().len(), 1);
    }
}
