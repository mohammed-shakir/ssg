use clap::Parser;
use std::{fs, path::PathBuf, thread, time::Duration};

#[test]
fn cli_parses_build_args() {
    let args = ssg::cli::Args::parse_from(["ssg", "build", "--src", "a", "--out", "b"]);
    match args.action {
        ssg::cli::Action::Build { src, out } => {
            assert_eq!(src, PathBuf::from("a"));
            assert_eq!(out, PathBuf::from("b"));
        }
        _ => panic!("expected build"),
    }
}

fn write_min_site(root: &std::path::Path) {
    fs::create_dir_all(root.join("templates")).unwrap();
    fs::write(
        root.join("templates/base.html"),
        "<!doctype html><title>{% block title %}{{ page.title }}{% endblock %}</title>{% block content %}{% endblock %}",
    ).unwrap();
    fs::write(
        root.join("templates/post.html"),
        r#"{% extends "base.html" %}{% block content %}<h1>{{ page.title }}</h1>{{ page.content | safe }}{% endblock %}"#,
    ).unwrap();
    fs::write(root.join("index.md"), "---\ntitle: Home\n---\n# Hello").unwrap();
    fs::create_dir_all(root.join("posts")).unwrap();
    fs::write(
        root.join("posts/first.md"),
        "---\ntitle: First\n---\n# First",
    )
    .unwrap();
    fs::write(
        root.join("site.toml"),
        r#"
            title = "T"
            base_url = "http://localhost/"
            src = "."
            out = "out"
        "#,
    )
    .unwrap();
}

#[test]
fn build_then_skip_when_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("site");
    let out = tmp.path().join("dist");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&out).unwrap();
    write_min_site(&src);

    let args = ssg::cli::Args {
        action: ssg::cli::Action::Build {
            src: src.clone(),
            out: out.clone(),
        },
    };
    ssg::run(args);

    let page = out.join("posts/first/index.html");
    let m1 = fs::metadata(&page).unwrap().modified().unwrap();

    let args2 = ssg::cli::Args {
        action: ssg::cli::Action::Build {
            src: src.clone(),
            out: out.clone(),
        },
    };
    ssg::run(args2);
    let m2 = fs::metadata(&page).unwrap().modified().unwrap();

    assert_eq!(m1, m2, "content page should be skipped");
    assert!(out.join(".ssg-cache.json").exists());
}

#[test]
fn template_change_triggers_rebuild() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("site");
    let out = tmp.path().join("dist");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&out).unwrap();
    write_min_site(&src);

    ssg::run(ssg::cli::Args {
        action: ssg::cli::Action::Build {
            src: src.clone(),
            out: out.clone(),
        },
    });
    let page = out.join("posts/first/index.html");
    let m1 = fs::metadata(&page).unwrap().modified().unwrap();

    thread::sleep(Duration::from_millis(1100));
    fs::write(
        src.join("templates/base.html"),
        "<!doctype html>CHANGED{% block content %}{% endblock %}",
    )
    .unwrap();

    ssg::run(ssg::cli::Args {
        action: ssg::cli::Action::Build {
            src: src.clone(),
            out: out.clone(),
        },
    });
    let m2 = fs::metadata(&page).unwrap().modified().unwrap();

    assert!(m2 > m1, "page should be rebuilt after template change");
}
