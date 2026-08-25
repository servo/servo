// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: Simple fixed-length matches
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

assert.sameValue("b".match(/^.(?<=a)/), null, "#1");
assert.sameValue("boo".match(/^f\w\w(?<=\woo)/), null, "#2");
assert.sameValue("fao".match(/^f\w\w(?<=\woo)/), null, "#3");
assert.sameValue("foa".match(/^f\w\w(?<=\woo)/), null, "#4");

assert.compareArray("a".match(/^.(?<=a)/), ["a"], "#5");
assert.compareArray("foo1".match(/^f..(?<=.oo)/), ["foo"], "#6");
assert.compareArray("foo2".match(/^f\w\w(?<=\woo)/), ["foo"], "#7");
assert.compareArray("abcdef".match(/(?<=abc)\w\w\w/), ["def"], "#8");
assert.compareArray("abcdef".match(/(?<=a.c)\w\w\w/), ["def"], "#9");
assert.compareArray("abcdef".match(/(?<=a\wc)\w\w\w/), ["def"], "#10");
assert.compareArray("abcdef".match(/(?<=a[a-z])\w\w\w/), ["cde"], "#11");
assert.compareArray("abcdef".match(/(?<=a[a-z][a-z])\w\w\w/), ["def"], "#12");
assert.compareArray("abcdef".match(/(?<=a[a-z]{2})\w\w\w/), ["def"], "#13");
assert.compareArray("abcdef".match(/(?<=a{1})\w\w\w/), ["bcd"], "#14");
assert.compareArray("abcdef".match(/(?<=a{1}b{1})\w\w\w/), ["cde"], "#15");
assert.compareArray("abcdef".match(/(?<=a{1}[a-z]{2})\w\w\w/), ["def"], "#16");
