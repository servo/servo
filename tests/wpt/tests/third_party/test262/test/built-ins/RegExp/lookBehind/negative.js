// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: RegExp negative lookbehind
info: |
  The production Assertion :: (?<!Disjunction) evaluates as follows:
    1. Evaluate Disjunction with -1 as its direction argument to obtain a Matcher m.
    2. Return an internal Matcher closure that takes two arguments, a State x and a
        Continuation c, and performs the following steps:
      a. Let d be a Continuation that always returns its State argument as a successful
          MatchResult.
      b. Call m(x, d) and let r be its result.
      c. If r is not failure, return failure.
      d. Call c(x) and return its result.
features: [regexp-lookbehind]
includes: [compareArray.js]
---*/

assert.compareArray("abcdef".match(/(?<!abc)\w\w\w/), ["abc"], "#1");
assert.compareArray("abcdef".match(/(?<!a.c)\w\w\w/), ["abc"], "#2");
assert.compareArray("abcdef".match(/(?<!a\wc)\w\w\w/), ["abc"], "#3");
assert.compareArray("abcdef".match(/(?<!a[a-z])\w\w\w/), ["abc"], "#4");
assert.compareArray("abcdef".match(/(?<!a[a-z]{2})\w\w\w/), ["abc"], "#5");
assert.sameValue("abcdef".match(/(?<!abc)def/), null, "#6");
assert.sameValue("abcdef".match(/(?<!a.c)def/), null, "#7");
assert.sameValue("abcdef".match(/(?<!a\wc)def/), null, "#8");
assert.sameValue("abcdef".match(/(?<!a[a-z][a-z])def/), null, "#9");
assert.sameValue("abcdef".match(/(?<!a[a-z]{2})def/), null, "#10");
assert.sameValue("abcdef".match(/(?<!a{1}b{1})cde/), null, "#11");
assert.sameValue("abcdef".match(/(?<!a{1}[a-z]{2})def/), null, "#12");
