// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomescape
es6id: 21.2.2.9
description: >
  Support for surrogate pairs within patterns match by the DecimalEscape
  AtomEscape
info: |
  The production AtomEscape :: DecimalEscape evaluates as follows:

  [...]
  3. Return an internal Matcher closure that takes two arguments, a State x and
     a Continuation c, and performs the following steps:
     [...]
     h. If there exists an integer i between 0 (inclusive) and len (exclusive)
        such that Canonicalize(s[i]) is not the same character value as
        Canonicalize(Input[e+i]), return failure.

  Runtime Semantics: CharacterSetMatcher Abstract Operation

  1. Return an internal Matcher closure that takes two arguments, a State x and
     a Continuation c, and performs the following steps when evaluated:
     [...]
     d. Let cc be Canonicalize(ch).
     [...]
---*/

assert.sameValue(/(.+).*\1/u.test('\ud800\udc00\ud800'), false);
