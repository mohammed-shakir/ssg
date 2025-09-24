use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, SystemTime},
};

use mime_guess::from_path;
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use tiny_http::{Header, Response, Server};
use walkdir::WalkDir;

pub fn serve(src: &Path, out: &Path) {
    let manifest_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    let src = if src.is_absolute() {
        src.to_path_buf()
    } else {
        manifest_root.join(src)
    };
    let out = if out.is_absolute() {
        out.to_path_buf()
    } else {
        manifest_root.join(out)
    };

    let _ = fs::create_dir_all(&out);

    crate::build(&src, &out);

    let mut last_build = SystemTime::now();
    let (tx, rx) = mpsc::channel::<()>();
    let _watcher = spawn_watcher(src.clone(), out.clone(), tx.clone());
    let server_thread = spawn_http(out.clone());

    println!("Dev server: http://127.0.0.1:4000  (Ctrl+C to quit)");

    loop {
        if rx.recv().is_ok() {
            while rx.try_recv().is_ok() {}
            if !has_changes_since(&src, last_build) {
                continue;
            }
            println!("↻ Rebuilding…");
            crate::build(&src, &out);
            last_build = SystemTime::now();
            println!("✓ Rebuilt");
            std::thread::sleep(Duration::from_millis(100));
            while rx.try_recv().is_ok() {}
        } else {
            break;
        }
    }

    let _ = server_thread.join();
}

fn spawn_watcher(src_dir: PathBuf, out_dir: PathBuf, tx: mpsc::Sender<()>) -> Option<impl Drop> {
    let src_dir = std::fs::canonicalize(&src_dir).ok()?;
    let out_dir = std::fs::canonicalize(&out_dir).unwrap_or(out_dir);

    let src_dir_cb = src_dir.clone();
    let out_dir_cb = out_dir.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |res: DebounceEventResult| match res {
            Ok(events) => {
                let mut trigger = false;
                for e in events {
                    let Ok(p) = std::fs::canonicalize(&e.path) else {
                        continue;
                    };
                    if p.starts_with(&out_dir_cb) {
                        continue;
                    }
                    if !p.starts_with(&src_dir_cb) {
                        continue;
                    }
                    if let Some(name) = p.file_name().and_then(|n| n.to_str())
                        && (name.starts_with('.') || name.ends_with('~') || name.ends_with(".swp"))
                    {
                        continue;
                    }
                    let ok_ext = matches!(
                        p.extension().and_then(|e| e.to_str()),
                        Some("md" | "toml" | "html" | "css" | "js")
                    );
                    if !ok_ext {
                        continue;
                    }
                    trigger = true;
                    break;
                }
                if trigger {
                    let _ = tx.send(());
                }
            }
            Err(_) => {
                let _ = tx.send(());
            }
        },
    )
    .ok()?;

    if debouncer
        .watcher()
        .watch(&src_dir, RecursiveMode::Recursive)
        .is_err()
    {
        eprintln!("watch: path not found: {}", src_dir.display());
    }
    Some(debouncer)
}

fn spawn_http(out: PathBuf) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let server = Server::http("127.0.0.1:4000").expect("bind 127.0.0.1:4000");
        for rq in server.incoming_requests() {
            let url = rq.url().trim_start_matches('/');
            let mut path = out.join(url);
            if rq.url().ends_with('/') || is_dir(&path) {
                path = out.join(url).join("index.html");
            }
            match fs::File::open(&path) {
                Ok(file) => {
                    let mime = from_path(&path).first_or_octet_stream();
                    let hdr = Header::from_bytes(&b"Content-Type"[..], mime.as_ref()).unwrap();
                    let mut resp = Response::from_file(file);
                    resp.add_header(hdr);
                    let _ = rq.respond(resp);
                }
                Err(_) => {
                    let body = b"404 Not Found";
                    let _ = rq.respond(Response::from_data(body.as_slice()).with_status_code(404));
                }
            }
        }
    })
}

fn has_changes_since(root: &Path, since: SystemTime) -> bool {
    for entry in WalkDir::new(root) {
        let Ok(e) = entry else { continue };
        if !e.file_type().is_file() {
            continue;
        }
        let p = e.path();
        let ok_ext = matches!(
            p.extension().and_then(|e| e.to_str()),
            Some("md" | "toml" | "html" | "css" | "js")
        );
        if !ok_ext {
            continue;
        }
        if fs::metadata(p)
            .and_then(|m| m.modified())
            .is_ok_and(|m| m > since)
        {
            return true;
        }
    }
    false
}

fn is_dir(p: &Path) -> bool {
    fs::metadata(p).map(|m| m.is_dir()).unwrap_or(false)
}
