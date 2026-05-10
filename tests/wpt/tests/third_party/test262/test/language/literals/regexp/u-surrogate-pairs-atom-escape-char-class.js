// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomescape
es6id: 21.2.2.9
description: >
  Support for surrogate pairs within patterns match by the CharacterClassEscape
  AtomEscape
info: |
  The production AtomEscape :: CharacterClassEscape evaluates as follows:

  1. Evaluate CharacterClassEscape to obtain a CharSet A.
  2. Call CharacterSetMatcher(A, false) and return its Matcher result. 

  Runtime Semantics: CharacterSetMatcher Abstract Operation

  1. Return an internal Matcher closure that takes two arguments, a State x and
     a Continuation c, and performs the following steps when evaluated:
     [...]
     d. Let cc be Canonicalize(ch).
     [...]
---*/

assert(/^\S$/u.test('\ud800\udc00'));
