---
status: proposed
tags: [bootstrap, lsp]
created: 2026-10-07
created-by: claude
reviewed-by: ""
parent: ""
relations:
  implements: [lsp-binary-resolution]
---

# ADR: No standalone clj-kondo language server

Decision (commit 7bf4acf): the extension ships clojure-lsp only and
does not declare a clj-kondo language server.

Context: clj-kondo's upstream CHANGELOG (release 2026.05.25) says it
was the last release to include the clj-kondo LSP server, directing
editor integration to clojure-lsp, which embeds clj-kondo. Binaries
after that (e.g. Homebrew v2026.08.04) exit with help text on
`--lsp`, so the extension's clj-kondo server crashed on start.
Zed 1.22.0 also ignores `opt_in_languages`, so the previously opt-in
server auto-started and crashed for users who never asked for it.

Consequences: linting still arrives via clojure-lsp's embedded
clj-kondo; a future re-add would need a pinned download of the last
LSP-capable release (2026.05.25) and is not worth the maintenance.
