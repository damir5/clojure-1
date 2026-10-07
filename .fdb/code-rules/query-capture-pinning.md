---
status: proposed
tags: [bootstrap, queries]
created: 2026-10-07
created-by: claude
reviewed-by: ""
parent: ""
relations:
  verifies: [def-family-head]
---

# Query capture pinning

Rule: every query file must have a designated query × fixture pair in
`script/check-queries.sh` whose output is pinned to specific capture
names and captured symbols (e.g. `outline.scm` × `outline.clj` must
emit `@item` and capture the text `` `public-fn` ``, `` `private-fn` ``,
`` `Shape` ``). A new query file without pins fails review, not CI —
pins are listed explicitly in the `pins` heredoc.

Why: a tree-sitter query parses cleanly and still silently stops
matching a construct it was written for (a def head dropped from an
`#any-of?` list). Match-nothing checks can't see that; pins can.

Enforced by: the `pins` loop in `script/check-queries.sh` (CI job).

Anchor: `# @fdb:query-capture-pinning` belongs on the `pins=` block.
