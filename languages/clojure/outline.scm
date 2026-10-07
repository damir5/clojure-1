;; ns form

(list_lit
  .
  (sym_lit
    (sym_name) @_ns) @context
  .
  (sym_lit) @name
  (#eq? @_ns "ns")) @item

;; def-family: the name is the symbol (or keyword, for s/def) after the
;; head. Specifier forms like s/def and m/=> match on sym_name, so the
;; ns-qualified head shows up as context for free. defn- lives only in
;; the private pattern below to avoid duplicate items.

(list_lit
  .
  (sym_lit
    (sym_name) @_def-head) @context
  .
  [
    (sym_lit)
    (kwd_lit)
  ] @name
  (#any-of? @_def-head
    "def" "defonce" "declare" "defn" "definline" "defmacro"
    "defmulti" "deftest" "defprotocol" "defrecord" "deftype"
    "definterface" "fdef" "=>")) @item

;; defmethod: multi-name plus dispatch value's arg vector

(list_lit
  .
  (sym_lit
    (sym_name) @_defmethod) @context
  .
  (sym_lit) @name
  (vec_lit) @name
  (#eq? @_defmethod "defmethod")) @item

;; Method signatures nested in defprotocol/defrecord/deftype

(list_lit
  .
  (sym_lit
    (sym_name) @_def-type) @context
  .
  (sym_lit) @name
  (list_lit
    .
    (sym_lit) @name
    .
    (vec_lit)) @item
  (#any-of? @_def-type "defprotocol" "defrecord" "deftype")) @item

;; Top-level rich comment blocks

(list_lit
  .
  (sym_lit
    (sym_name) @_comment) @name
  (#eq? @_comment "comment")) @item

;; Private vars: defn- gets an extra context marker

(list_lit
  .
  (sym_lit
    (sym_name) @_private) @context.extra
  .
  (sym_lit) @name
  (#eq? @_private "defn-")) @item
