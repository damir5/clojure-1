---
status: proposed
tags: [bootstrap, lsp]
created: 2026-10-07
created-by: claude
reviewed-by: ""
parent: ""
relations:
  verifies: [no-clj-kondo-server]
---

# LSP binary resolution order

Rule: resolve a language-server binary in this order, first match
wins. `lsp.<name>.binary.path` setting, then `worktree.which(binary)`
(PATH), then a previously resolved path that still exists, then the
latest GitHub release downloaded into a `clojure-lsp-<version>`
directory with outdated versions of the same tool removed.

Why: PATH is honored so users can run their own build (Homebrew,
Nix), and the per-version cache plus scoped cleanup keeps upgrades
working without ever deleting another tool's cache.

Enforced by: unit tests on `resolve_binary_path` in `src/clojure.rs`
(path wins without a release lookup, cache short-circuit, download +
cleanup, existing download skips both, missing asset and unsupported
arch are errors).

Anchor: `// @fdb:lsp-binary-resolution` belongs on the
`resolve_binary_path` test module.
