// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: Mutual recursive capture/back references
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

assert.compareArray(/(?<=a(.\2)b(\1)).{4}/.exec("aabcacbc"), ["cacb", "a", ""], "#1");
assert.compareArray(/(?<=a(\2)b(..\1))b/.exec("aacbacb"), ["b", "ac", "ac"], "#2");
assert.compareArray(/(?<=(?:\1b)(aa))./.exec("aabaax"), ["x", "aa"], "#3");
assert.compareArray(/(?<=(?:\1|b)(aa))./.exec("aaaax"), ["x", "aa"], "#4");
