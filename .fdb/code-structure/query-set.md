---
status: proposed
tags: [bootstrap, queries]
created: 2026-10-07
created-by: claude
reviewed-by: ""
parent: ""
relations:
  depends-on: [query-capture-pinning]
---

# Query set (languages/clojure/*.scm)

The query set is eight tree-sitter query files against the pinned
sogaiu/tree-sitter-clojure grammar. The files are highlights
(Sublimed scope port), outline, textobjects, runnables, brackets,
indents, injections and overrides. They are checked by
`script/check-queries.sh` (also a CI job): every query runs against
every fixture in `test/fixtures/`, plus sanity pairs and capture pins.
`test/fixtures/syntax_test.cljc` is copied from Clojure Sublimed's test
suite and is the authoritative reader-syntax torture file.

Grammar bumps are the main risk: unknown node types fail the query
compile step, and scope drift fails the pins.
