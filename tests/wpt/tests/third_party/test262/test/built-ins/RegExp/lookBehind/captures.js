// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: >
  Capturing matches
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

var str = "abcdef";
assert.compareArray(str.match(/(?<=(c))def/), ["def", "c"], "#1");
assert.compareArray(str.match(/(?<=(\w{2}))def/), ["def", "bc"], "#2");
assert.compareArray(str.match(/(?<=(\w(\w)))def/), ["def", "bc", "c"], "#3");
assert.compareArray(str.match(/(?<=(\w){3})def/), ["def", "a"], "#4");
assert.compareArray(str.match(/(?<=(bc)|(cd))./), ["d", "bc", undefined], "#5");
assert.compareArray(str.match(/(?<=([ab]{1,2})\D|(abc))\w/), ["c", "a", undefined], "#6");
assert.compareArray(str.match(/\D(?<=([ab]+))(\w)/), ["ab", "a", "b"], "#7");
assert.compareArray(str.match(/(?<=b|c)\w/g), ["c", "d"], "#8");
assert.compareArray(str.match(/(?<=[b-e])\w{2}/g), ["cd", "ef"], "#9");
