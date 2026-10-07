# Changelog

## 0.3.0

- Grammar: pin sogaiu/tree-sitter-clojure at `e43eff8` (2025-08-26).
- Highlights: full port of Clojure Sublimed's scope decisions —
  discarded forms, rich comments, metadata, quoting markers, regex,
  namespaced keywords/symbols, def-family names, reader conditionals,
  tagged literals, var-quote/deref, brackets.
- Outline: def-family, defmethod (name + dispatch value), nested
  protocol/record/type method signatures, s/def, s/fdef, m/=>,
  top-level `(comment …)` blocks, private (`defn-`) marker.
- New `textobjects.scm` (af/if, ac/ic, gc in Vim mode).
- New `runnables.scm`: ▶ on `deftest` forms with a `clojure-test` tag.
- New `overrides.scm` + `[overrides.string]` completion settings.
- Config: `nbb`, `cljx`, `boot` suffixes; `; ` line comment;
  `kernel_language_names`; `auto_indent_using_last_non_empty_line`.
- clj-kondo dropped as a language server: upstream removed its LSP mode
  after release 2026.05.25, so the server crashed on start; clojure-lsp
  embeds clj-kondo and covers linting.
- Binary resolution reworked behind a testable seam: `lsp.<name>.binary.path`
  → worktree PATH → cached download → latest GitHub release, with unit
  tests for the order and for cleanup of outdated versions.
- LSP settings passthrough (`initialization_options`, `settings`) and
  polished completion labels.
- Formatting recipe: [docs/formatting.md](docs/formatting.md) plus a
  `cljfmt-tonsky` snippet.
- CI: query checks against all fixtures, clippy with `-D warnings`.
- Post-review fixes: nested
  protocol/record/type method items no longer concatenate the type
  name into their labels; `defmethod` outline handles multi-arity
  bodies and vector dispatch values; `#?@` branch keys highlighted;
  `lsp.<name>.binary.path` settings are honored (with user arguments
  and env); deftest runnables expose `$ZED_CUSTOM_test_name` without
  trailing metadata; quote autoclose disabled inside comments; Cargo
  version synced to 0.3.0.
