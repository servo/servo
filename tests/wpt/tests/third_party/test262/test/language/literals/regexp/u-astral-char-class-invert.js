// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atom
description: >
  Literal astral symbols within inverted CharacterClass.
info: |
  The production Atom :: CharacterClass evaluates as follows:

  1. Evaluate CharacterClass to obtain a CharSet A and a Boolean invert.
  2. Call CharacterSetMatcher(A, invert, direction) and return its Matcher result.

  Runtime Semantics: CharacterSetMatcher ( A, invert, direction )

  1. Return an internal Matcher closure that takes two arguments, a State x and
  a Continuation c, and performs the following steps:
    [...]
    f. Let cc be Canonicalize(ch).
    g. If invert is false, then
      [...]
    h. Else,
      i. Assert: invert is true.
      ii. If there exists a member a of set A such that Canonicalize(a) is cc,
      return failure.
---*/

assert.sameValue(/^[^❤️]$/u.exec("❤️"), null);
assert.sameValue(/^[^🧡]/u.exec("🧡"), null);
assert.sameValue(/[^💛]$/u.exec("💛"), null);
assert.sameValue(/[^💚]/u.exec("💚"), null);
