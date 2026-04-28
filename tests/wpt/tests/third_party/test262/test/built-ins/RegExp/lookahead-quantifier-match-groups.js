// Copyright (C) 2023 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-repeatmatcher-abstract-operation
description: 0-length matches update the captures list if and only if they are not followed by a quantifier which allows them to match 0 times.
info: |
  RepeatMatcher
    1. If max = 0, return c(x).
    2. Let d be a new MatcherContinuation with parameters (y) that captures m, min, max, greedy, x, c, parenIndex, and parenCount and performs the following steps when called:
      [...]
      b. If min = 0 and y's endIndex = x's endIndex, return failure.
      c. If min = 0, let min2 be 0; otherwise let min2 be min - 1.
      d. If max = +∞, let max2 be +∞; otherwise let max2 be max - 1.
      e. Return RepeatMatcher(m, min2, max2, greedy, y, c, parenIndex, parenCount).
    3. Let cap be a copy of x's captures List.
    4. For each integer k in the inclusive interval from parenIndex + 1 to parenIndex + parenCount, set cap[k] to undefined.
    [...]
    7. Let xr be the MatchState (Input, e, cap).
    [...]
    10. Let z be m(xr, d).
    11. If z is not failure, return z.
    12. Return c(x).
includes: [compareArray.js]
---*/

assert.compareArray("abc".match(/(?:(?=(abc)))a/), ["a", "abc"], "unquantified");
assert.compareArray("abc".match(/(?:(?=(abc)))?a/), ["a", undefined], "? quantifier");
assert.compareArray("abc".match(/(?:(?=(abc))){1,1}a/), ["a", "abc"], "{1,1} quantifier");
assert.compareArray("abc".match(/(?:(?=(abc))){0,1}a/), ["a", undefined], "{0,1} quantifier");
