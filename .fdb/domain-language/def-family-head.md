---
status: proposed
tags: [bootstrap, queries]
created: 2026-10-07
created-by: claude
reviewed-by: ""
parent: ""
relations:
  verifies: [query-capture-pinning]
---

# Def-family head

The def-family heads are the Clojure defining forms that get special
treatment across the query files. The set: `def`, `defonce`,
`declare`, `defn`, `defn-`, `defmacro`, `definline`, `defmulti`,
`defmethod`, `deftest`, `defprotocol`, `defrecord`, `deftype`,
`definterface`. Each query file
(highlights, outline, textobjects) tracks its own subset via its own
`#any-of?` list — tree-sitter queries cannot share lists — so the
coverage can drift silently. The capture pins in
`script/check-queries.sh` are the guard: dropping a head like `defn`
from `outline.scm` fails CI, not just silently matches less.

Related: [[rich-comment-block]]
