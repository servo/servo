// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: Sticky
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

var re1 = /(?<=^(\w+))def/g;
assert.compareArray(re1.exec("abcdefdef"), ["def", "abc"], "#1");
assert.compareArray(re1.exec("abcdefdef"), ["def", "abcdef"], "#2");

var re2 = /\Bdef/g;
assert.compareArray(re2.exec("abcdefdef"), ["def"], "#3");
assert.compareArray(re2.exec("abcdefdef"), ["def"], "#4");
