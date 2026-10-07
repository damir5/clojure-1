---
status: proposed
tags: [bootstrap, queries]
created: 2026-10-07
created-by: claude
reviewed-by: ""
parent: ""
relations:
  implements: [query-set]
---

# ADR: Port highlighting scopes from Clojure Sublimed

Decision (commits 84a4e49, 0afd09f): port Clojure Sublimed's scope
decisions instead of inventing a scope mapping from scratch. The port
covers discarded forms, rich comments, metadata, quoting markers,
namespaced keywords/symbols, reader-conditional branch keys, tagged
literals. `test/fixtures/syntax_test.cljc` is copied from Sublimed's
test suite as the shared reference.

Consequences: highlighting behavior tracks a respected external
reference instead of ad-hoc taste, and upgrades can be diffed against
upstream Sublimed tests. Divergence from Sublimed is deliberate only
where Zed's capture vocabulary differs.
