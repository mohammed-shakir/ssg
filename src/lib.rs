pub mod cli;
pub mod config;
pub mod content;
pub mod render;
pub mod routing;
pub mod taxonomy;
pub mod templates;

use crate::{
    cli::{Action, Args},
    config::{SiteConfig, load_config},
    content::{PageMeta, collect_markdown_files, load_document},
    routing::{copy_static_assets, out_path_for},
    taxonomy::{PageSummary, summarize, write_tag_pages},
    templates::Templates,
};
use std::{fs, path::Path};

pub fn run(args: Args) {
    match args.action {
        Action::Build { src, out } => build(&src, &out),
        Action::Serve { out } => serve(&out),
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
    for md in md_files {
        let doc = match load_document::<PageMeta>(&md) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("load {}: {e}", md.display());
                continue;
            }
        };

        let html = match templates.render_page(&cfg, &doc) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("render {}: {e}", md.display());
                continue;
            }
        };

        let out_path = out_path_for(&cfg.src_dir, &cfg.out_dir, &md, &doc);
        if let Some(parent) = out_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&out_path, &html) {
            eprintln!("write {}: {e}", out_path.display());
            continue;
        }

        let meta = doc.front_matter.clone().unwrap_or_default();
        let title = meta.title.as_deref().unwrap_or("Untitled");
        summaries.push(summarize(&doc, &cfg.out_dir, &out_path, &meta.tags, title));
    }

    if let Err(e) = copy_static_assets(&cfg.src_dir, &cfg.out_dir) {
        eprintln!("assets: {e}");
    }

    if let Err(e) = write_tag_pages(&templates, &cfg, &cfg.out_dir, &summaries) {
        eprintln!("tags: {e}");
    }
}

fn serve(out: &Path) {
    println!("Serve from {:?}", out);
}

fn clean(out: &Path) {
    println!("Clean {:?}", out);
}
