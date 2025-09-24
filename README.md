# Static site generator (SSG)

A simple static site generator (SSG) built in rust.

## Run

```bash
# build the demo site into ./dist
cargo run -- build --src demo_site --out dist

# dev server with watch on http://127.0.0.1:4000
cargo run -- serve --src demo_site --out dist
