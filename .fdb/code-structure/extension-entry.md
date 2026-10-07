---
status: proposed
tags: [bootstrap, lsp]
created: 2026-10-07
created-by: claude
reviewed-by: ""
parent: ""
relations:
  depends-on: [lsp-binary-resolution]
---

# Extension entry (src/clojure.rs)

The whole Rust→WASM extension is one file, `src/clojure.rs` (target
`wasm32-wasip2`, zed_extension_api 0.7). Three parts:

- `ToolSpec` — data describing one downloadable server tool (repo,
  binary name, asset prefix). Owns asset naming and the per-version
  install paths, so adding a server is a data change. Currently one
  instance: `CLOJURE_LSP`.
- `Env` trait — the seam between the binary-resolution policy and the
  outside world (platform, GitHub releases, filesystem, downloader).
  `ZedEnv` wires the real Zed API; the unit tests wire fakes and
  exercise the order without network or disk.
- `ClojureExtension` — the `zed::Extension` impl: server command,
  settings passthrough (`initialization_options`, `settings`), and
  completion labels.

`extension.toml` declares the language server, grammar pin and the
`languages/clojure/` assets.
