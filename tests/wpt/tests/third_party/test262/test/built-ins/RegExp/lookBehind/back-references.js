// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: Back references
info: |
  The production Assertion :: (?<=Disjunction) evaluates as follows:
    1. Evaluate Disjunction with -1 as its direction argument to obtain a Matcher m.
    2. Return an internal Matcher closure that takes two arguments, a State x and a Continuation
        c, and performs the following steps:
      a. Let d be a Continuation that always returns its State argument as a successful MatchResult.
      b. Call m(x, d) and let r be its result.
      c. If r is failure, return failure.
      d. Let y be r's State.
      e. Let cap be y's captures List.
      f. Let xe be x's endIndex.
      g. Let z be the State (xe, cap).
      h. Call c(z) and return its result.
features: [regexp-lookbehind]
includes: [compareArray.js]
---*/

assert.compareArray("abb".match(/(.)(?<=(\1\1))/), ["b", "b", "bb"], "#1");
assert.compareArray("abB".match(/(.)(?<=(\1\1))/i), ["B", "B", "bB"], "#2");
assert.compareArray("aabAaBa".match(/((\w)\w)(?<=\1\2\1)/i), ["aB", "aB", "a"], "#3");
assert.compareArray("aabAaBa".match(/(\w(\w))(?<=\1\2\1)/i), ["Ba", "Ba", "a"], "#4");
assert.compareArray("abaBbAa".match(/(?=(\w))(?<=(\1))./i), ["b", "b", "B"], "#5");
assert.compareArray("  'foo'  ".match(/(?<=(.))(\w+)(?=\1)/), ["foo", "'", "foo"], "#6");
assert.compareArray("  \"foo\"  ".match(/(?<=(.))(\w+)(?=\1)/), ["foo", "\"", "foo"], "#7");
assert.compareArray("abbb".match(/(.)(?<=\1\1\1)/), ["b", "b"], "#8");
assert.compareArray("fababab".match(/(..)(?<=\1\1\1)/), ["ab", "ab"], "#9");

assert.sameValue("  .foo\"  ".match(/(?<=(.))(\w+)(?=\1)/), null, "#10");
assert.sameValue("ab".match(/(.)(?<=\1\1\1)/), null, "#11");
assert.sameValue("abb".match(/(.)(?<=\1\1\1)/), null, "#12");
assert.sameValue("ab".match(/(..)(?<=\1\1\1)/), null, "#13");
assert.sameValue("abb".match(/(..)(?<=\1\1\1)/), null, "#14");
assert.sameValue("aabb".match(/(..)(?<=\1\1\1)/), null, "#15");
assert.sameValue("abab".match(/(..)(?<=\1\1\1)/), null, "#16");
assert.sameValue("fabxbab".match(/(..)(?<=\1\1\1)/), null, "#17");
assert.sameValue("faxabab".match(/(..)(?<=\1\1\1)/), null, "#18");

