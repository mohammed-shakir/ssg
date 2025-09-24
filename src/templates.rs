use crate::{
    config::SiteConfig,
    content::{Document, PageMeta},
    render::render_html_sanitized,
};
use serde::Serialize;
use std::{io, path::Path};
use tera::{Context, Tera};

pub struct Templates {
    tera: Tera,
}

impl Templates {
    pub fn load_from(dir: &Path) -> io::Result<Self> {
        let pattern = format!("{}/**/*", dir.display());
        let tera = Tera::new(&pattern).map_err(map_tera_err)?;
        Ok(Self { tera })
    }

    pub fn render_page(&self, cfg: &SiteConfig, doc: &Document<PageMeta>) -> io::Result<String> {
        let body_html = render_html_sanitized(doc);

        #[derive(Serialize)]
        struct SiteView<'a> {
            title: &'a str,
            base_url: &'a str,
            theme: &'a str,
            description: &'a Option<String>,
            author: &'a Option<String>,
        }

        #[derive(Serialize)]
        struct PageView<'a> {
            title: &'a str,
            slug: &'a str,
            tags: &'a [String],
            date: &'a Option<String>,
            draft: bool,
            content: &'a str,
        }

        let meta = doc.front_matter.clone().unwrap_or_default();
        let title = meta.title.as_deref().unwrap_or("Untitled");
        let slug_owned = meta.slug.clone().unwrap_or_else(|| {
            doc.path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("index")
                .to_string()
        });

        let site = SiteView {
            title: &cfg.title,
            base_url: &cfg.base_url,
            theme: &cfg.theme,
            description: &cfg.description,
            author: &cfg.author,
        };

        let page = PageView {
            title,
            slug: &slug_owned,
            tags: &meta.tags,
            date: &meta.date,
            draft: meta.draft,
            content: &body_html,
        };

        let mut ctx = Context::new();
        ctx.insert("site", &site);
        ctx.insert("page", &page);

        let tpl = meta.template.as_deref().unwrap_or("post.html");

        self.tera.render(tpl, &ctx).map_err(map_tera_err)
    }

    #[allow(dead_code)]
    pub fn full_reload(&mut self) -> io::Result<()> {
        self.tera.full_reload().map_err(map_tera_err)
    }

    pub fn render_with<T: serde::Serialize>(
        &self,
        template: &str,
        site: &crate::config::SiteConfig,
        key: &str,
        data: &T,
    ) -> std::io::Result<String> {
        use tera::Context;
        let mut ctx = Context::new();
        ctx.insert("site", site);
        ctx.insert(key, data);
        self.tera
            .render(template, &ctx)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn tera(&self) -> &tera::Tera {
        &self.tera
    }
}

fn map_tera_err(err: tera::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn renders_post_with_vars() -> io::Result<()> {
        let tmp = tempdir()?;
        let tpldir = tmp.path().join("templates");
        fs::create_dir_all(&tpldir)?;

        fs::write(
            tpldir.join("base.html"),
            r#"<!doctype html>
            <title>{% block title %}{{ page.title }} — {{ site.title }}{% endblock title %}</title>
            <main>{% block content %}{% endblock content %}</main>"#,
        )?;
        fs::write(
            tpldir.join("post.html"),
            r#"{% extends "base.html" %}{% block content %}
            <h1>{{ page.title }}</h1><div>{{ page.content | safe }}</div>
            {% endblock content %}"#,
        )?;

        let cfg = SiteConfig {
            title: "My Blog".into(),
            ..Default::default()
        };
        let doc = Document {
            path: tmp.path().join("hello.md"),
            front_matter: Some(PageMeta {
                title: Some("Hello".into()),
                ..Default::default()
            }),
            body: "# Hi\nBody".into(),
        };

        let t = Templates::load_from(&tpldir)?;
        let html = t.render_page(&cfg, &doc)?;

        assert!(html.contains("<title>Hello — My Blog</title>"));
        assert!(html.contains("<h1>Hello</h1>"));
        Ok(())
    }
}
