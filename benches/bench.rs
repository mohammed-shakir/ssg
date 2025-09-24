use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use ssg::content::{Document, PageMeta};
use ssg::render::render_html;
use std::{fs, path::PathBuf};
use tempfile::tempdir;

fn bench_render_markdown(c: &mut Criterion) {
    let body = r#"
# Title

- a
- b
- c

|h|h|
|-|-|
|a|b|

Footnote.[^1]

[^1]: hi
"#;
    let doc = Document::<PageMeta> {
        path: PathBuf::from("x.md"),
        front_matter: None,
        body: body.into(),
    };
    c.bench_function("render_markdown", |b| b.iter(|| render_html(&doc)));
}

fn bench_templates_hash(c: &mut Criterion) {
    c.bench_function("templates_hash_small", |b| {
        b.iter_batched(
            || {
                let tmp = tempdir().unwrap();
                let dir = tmp.path().join("templates");
                fs::create_dir_all(&dir).unwrap();
                fs::write(dir.join("a.html"), "A").unwrap();
                fs::write(dir.join("b.html"), "B").unwrap();
                (tmp, dir)
            },
            |(_, dir)| {
                let _ = ssg::cache::templates_hash(&dir).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_render_markdown, bench_templates_hash);
criterion_main!(benches);
