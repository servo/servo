// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assertion
description: Start of line matches
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

assert.sameValue("abcdef".match(/(?<=^[^a-c]{3})def/), null, "#1");
assert.sameValue("foooo".match(/"^foooo(?<=^o+)$/), null, "#2");
assert.sameValue("foooo".match(/"^foooo(?<=^o*)$/), null, "#3");

assert.compareArray("abcdef".match(/(?<=^abc)def/), ["def"], "#4");
assert.compareArray("abcdef".match(/(?<=^[a-c]{3})def/), ["def"], "#5");
assert.compareArray("xyz\nabcdef".match(/(?<=^[a-c]{3})def/m), ["def"], "#6");
assert.compareArray("ab\ncd\nefg".match(/(?<=^)\w+/gm), ["ab", "cd", "efg"], "#7");
assert.compareArray("ab\ncd\nefg".match(/\w+(?<=$)/gm), ["ab", "cd", "efg"], "#8");
assert.compareArray("ab\ncd\nefg".match(/(?<=^)\w+(?<=$)/gm), ["ab", "cd", "efg"], "#9");

assert.compareArray("foo".match(/^foo(?<=^fo+)$/), ["foo"], "#10");
assert.compareArray("foooo".match(/^foooo(?<=^fo*)/), ["foooo"], "#11");
assert.compareArray("foo".match(/^(f)oo(?<=^\1o+)$/), ["foo", "f"], "#12");
assert.compareArray("foo".match(/^(f)oo(?<=^\1o+)$/i), ["foo", "f"], "#13");
assert.compareArray("foo\u1234".match(/^(f)oo(?<=^\1o+).$/i), ["foo\u1234", "f"], "#14");
assert.compareArray("abcdefdef".match(/(?<=^\w+)def/), ["def"], "#15");
assert.compareArray("abcdefdef".match(/(?<=^\w+)def/g), ["def", "def"], "#16");
