;; Scopes where editing behaves differently from plain code; see the
;; [overrides.*] tables in config.toml.

[
  (str_lit)
  (regex_lit)
] @string

(comment) @comment.inclusive

(dis_expr) @comment.inclusive
