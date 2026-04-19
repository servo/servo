// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: Word boundary
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

// Word boundary matches.
assert.compareArray("abc def".match(/(?<=\b)[d-f]{3}/), ["def"], "#1");
assert.compareArray("ab cdef".match(/(?<=\B)\w{3}/), ["def"], "#2");
assert.compareArray("ab cdef".match(/(?<=\B)(?<=c(?<=\w))\w{3}/), ["def"], "#3");
assert.sameValue("abcdef".match(/(?<=\b)[d-f]{3}/), null, "#4");
