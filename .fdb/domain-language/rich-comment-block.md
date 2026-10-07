---
status: proposed
tags: [bootstrap, queries]
created: 2026-10-07
created-by: claude
reviewed-by: ""
parent: ""
relations:
  relates-to: [def-family-head]
---

# Rich comment block

A top-level `(comment …)` form. Unlike discarded forms (`#_`), its
inner forms keep their normal highlighting (only the head symbol gets
the doc-comment scope), and the block itself is an outline item so
repl-style experimentation files stay navigable. Inner definitions are
deliberately not outline items — only the block is. Ported from
Clojure Sublimed's scope decisions.
