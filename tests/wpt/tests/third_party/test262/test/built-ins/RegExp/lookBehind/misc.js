// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: Misc RegExp lookbehind tests
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

assert.sameValue("abcdef".match(/(?<=$abc)def/), null, "#1");
assert.sameValue("fno".match(/^f.o(?<=foo)$/), null, "#2");
assert.sameValue("foo".match(/^foo(?<!foo)$/), null, "#3");
assert.sameValue("foo".match(/^f.o(?<!foo)$/), null, "#4");

assert.compareArray("foo".match(/^foo(?<=foo)$/), ["foo"], "#5");
assert.compareArray("foo".match(/^f.o(?<=foo)$/), ["foo"], "#6");
assert.compareArray("fno".match(/^f.o(?<!foo)$/), ["fno"], "#7");
assert.compareArray("foooo".match(/^foooo(?<=fo+)$/), ["foooo"], "#8");
assert.compareArray("foooo".match(/^foooo(?<=fo*)$/), ["foooo"], "#9");
assert.compareArray(/(abc\1)/.exec("abc"), ["abc", "abc"], "#10");
assert.compareArray(/(abc\1)/.exec("abc\u1234"), ["abc", "abc"], "#11");
assert.compareArray(/(abc\1)/i.exec("abc"), ["abc", "abc"], "#12");
assert.compareArray(/(abc\1)/i.exec("abc\u1234"), ["abc", "abc"], "#13");
