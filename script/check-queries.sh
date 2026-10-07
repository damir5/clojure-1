#!/usr/bin/env bash
# Compile the pinned tree-sitter-clojure grammar and run every query
# file against every fixture. Fails on any query error (unknown node
# types are the common failure after a grammar bump).
#
# Usage: script/check-queries.sh
# Requires: git, a C compiler, tree-sitter-cli >= 0.25.

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
grammar_dir="${TREE_SITTER_CLOJURE_DIR:-$root/../tree-sitter-clojure}"
commit="$(grep -A2 '\[grammars.clojure\]' "$root/extension.toml" | grep commit | cut -d'"' -f2)"

if [ ! -d "$grammar_dir" ]; then
  git clone --quiet https://github.com/sogaiu/tree-sitter-clojure "$grammar_dir"
fi
git -C "$grammar_dir" fetch --quiet origin
git -C "$grammar_dir" checkout --quiet "$commit"

lib="$grammar_dir/tree-sitter-clojure.so"
echo "Building grammar at $(git -C "$grammar_dir" rev-parse --short HEAD)"
tree-sitter build -o "$lib" "$grammar_dir"

fail=0
for query in "$root"/languages/clojure/*.scm; do
  for fixture in "$root"/test/fixtures/*; do
    if ! out=$(tree-sitter query --lib-path "$lib" --lang-name clojure "$query" "$fixture" 2>&1); then
      echo "FAIL: $(basename "$query") × $(basename "$fixture")"
      echo "$out" | head -5
      fail=1
    fi
  done
  echo "ok: $(basename "$query")"
done

# A query can parse cleanly and still match nothing, so require at
# least one capture for a designated query × fixture pair.
sanity='brackets.scm outline.clj
highlights.scm syntax_test.cljc
indents.scm indent.clj
injections.scm syntax_test.cljc
outline.scm outline.clj
overrides.scm edn_sample.edn
runnables.scm outline.clj
textobjects.scm outline.clj'
while read -r query fixture; do
  if [ -z "$(tree-sitter query --lib-path "$lib" --lang-name clojure \
      "$root/languages/clojure/$query" "$root/test/fixtures/$fixture")" ]; then
    echo "FAIL: $query matched nothing in $fixture"
    fail=1
  else
    echo "matched: $query × $fixture"
  fi
done <<<"$sanity"

# Capture pinning: a query can still match something yet silently stop
# matching a construct it was written for (e.g. a def head dropped from
# an #any-of? list). Each line names a query, a fixture and a string
# that MUST appear in that pair's output: a capture name or the quoted
# text of a captured node.
pins='brackets.scm outline.clj capture: open,
highlights.scm syntax_test.cljc capture: punctuation.bracket,
highlights.scm syntax_test.cljc capture: preproc,
indents.scm indent.clj capture: indent,
injections.scm syntax_test.cljc capture: content,
outline.scm outline.clj capture: item,
outline.scm outline.clj capture: context.extra,
outline.scm outline.clj text: `public-fn`
outline.scm outline.clj text: `private-fn`
outline.scm outline.clj text: `Shape`
outline.scm outline.clj text: `render`
outline.scm outline.clj text: `dispatch`
overrides.scm edn_sample.edn capture: string,
runnables.scm outline.clj capture: test_name,
runnables.scm outline.clj text: `meta-test`
textobjects.scm outline.clj capture: function.around,
textobjects.scm outline.clj capture: class.around,
textobjects.scm outline.clj capture: comment.around,'
while read -r query fixture pattern; do
  [ -z "$query" ] && continue
  out=$(tree-sitter query --lib-path "$lib" --lang-name clojure \
      "$root/languages/clojure/$query" "$root/test/fixtures/$fixture" 2>/dev/null)
  ok=0
  if [[ "$pattern" == capture:* ]]; then
    # Query output prints captures as `capture: name,` or, when a
    # pattern repeats a capture, `capture: N - name,`.
    regex="${pattern/capture: /capture: ([0-9]+ - )?}"
    grep -Eq -- "$regex" <<<"$out" && ok=1
  else
    grep -qF -- "$pattern" <<<"$out" && ok=1
  fi
  if [ "$ok" -eq 0 ]; then
    echo "FAIL: $query no longer matches [$pattern] in $fixture"
    fail=1
  else
    echo "pinned: $query × $fixture [$pattern]"
  fi
done <<<"$pins"

exit $fail

exit $fail
