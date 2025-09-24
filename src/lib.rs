pub mod cache;
pub mod cli;
pub mod config;
pub mod content;
pub mod devserver;
pub mod paginate;
pub mod render;
pub mod routing;
pub mod taxonomy;
pub mod templates;

use crate::{
    cache::BuildCache,
    cli::{Action, Args},
    config::{SiteConfig, load_config},
    content::{PageMeta, collect_markdown_files, load_document},
    routing::{copy_static_assets, out_path_for},
    taxonomy::{PageSummary, summarize, write_tag_pages},
    templates::Templates,
};
use rayon::prelude::*;
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

pub fn run(args: Args) {
    match args.action {
        Action::Build { src, out } => build(&src, &out),
        Action::Serve { src, out } => devserver::serve(&src, &out),
        Action::Clean { out } => clean(&out),
    }
}

fn build(src: &Path, out: &Path) {
    let cfg: SiteConfig = match load_config(src) {
        Ok(mut c) => {
            c.src_dir = src.to_path_buf();
            c.out_dir = out.to_path_buf();
            c
        }
        Err(e) => {
            eprintln!("config: {e}");
            return;
        }
    };

    let tpl_dir = cfg.src_dir.join("templates");
    let templates = match Templates::load_from(&tpl_dir) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("templates: {e}");
            return;
        }
    };

    let mut summaries: Vec<PageSummary> = Vec::new();
    let md_files = collect_markdown_files(&cfg.src_dir);

    let mut cache_prev: BuildCache = cache::load(&cfg.out_dir);
    let tpl_hash = cache::templates_hash(&tpl_dir).unwrap_or_default();
    let templates_changed = cache_prev.templates_hash != tpl_hash;

    let built = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    let prev_map = cache_prev.pages.clone();

    let results: Vec<PageSummary> = md_files
        .par_iter()
        .filter_map(|md| {
            let rel = md
                .strip_prefix(&cfg.src_dir)
                .unwrap_or(md)
                .to_string_lossy()
                .to_string();
            let file_hash = match cache::file_hash(md) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("hash {}: {e}", md.display());
                    return None;
                }
            };

            let up_to_date =
                !templates_changed && prev_map.get(&rel).is_some_and(|h| h == &file_hash);

            let doc = match load_document::<PageMeta>(md) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("load {}: {e}", md.display());
                    return None;
                }
            };

            let out_path = out_path_for(&cfg.src_dir, &cfg.out_dir, md, &doc);

            if up_to_date {
                skipped.fetch_add(1, Ordering::Relaxed);
            } else {
                let html = match templates.render_page(&cfg, &doc) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("render {}: {e}", md.display());
                        return None;
                    }
                };
                if let Some(parent) = out_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Err(e) = fs::write(&out_path, &html) {
                    eprintln!("write {}: {e}", out_path.display());
                    return None;
                }
                built.fetch_add(1, Ordering::Relaxed);
            }

            let meta = doc.front_matter.clone().unwrap_or_default();
            let title = meta.title.as_deref().unwrap_or("Untitled");
            Some(summarize(&doc, &cfg.out_dir, &out_path, &meta.tags, title))
        })
        .collect();

    summaries.extend(results);

    if let Err(e) = copy_static_assets(&cfg.src_dir, &cfg.out_dir) {
        eprintln!("assets: {e}");
    }
    if let Err(e) = write_tag_pages(&templates, &cfg, &cfg.out_dir, &summaries) {
        eprintln!("tags: {e}");
    }

    let mut new_pages = std::collections::HashMap::new();
    for md in &md_files {
        let rel = md
            .strip_prefix(&cfg.src_dir)
            .unwrap_or(md)
            .to_string_lossy()
            .to_string();
        if let Ok(h) = cache::file_hash(md) {
            new_pages.insert(rel, h);
        }
    }
    cache_prev.templates_hash = tpl_hash;
    cache_prev.pages = new_pages;
    if let Err(e) = cache::save(&cfg.out_dir, &cache_prev) {
        eprintln!("cache: {e}");
    }

    let b = built.load(Ordering::Relaxed);
    let s = skipped.load(Ordering::Relaxed);
    println!("Build done: {b} built, {s} skipped");
}

fn clean(out: &Path) {
    println!("Clean {:?}", out);
}
