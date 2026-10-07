;; deftest forms: run button on the test name.
;; The tag matches a user-defined task template (see README).

((list_lit
  .
  (sym_lit
    (sym_name) @_deftest)
  .
  (sym_lit) @run @test-name
  (#eq? @_deftest "deftest")) @_
  (#set! tag clojure-test))
