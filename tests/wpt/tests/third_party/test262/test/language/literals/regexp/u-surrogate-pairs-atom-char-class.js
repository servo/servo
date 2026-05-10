// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atom
es6id: 21.2.2.8
description: >
  Support for surrogate pairs within patterns match by the CharacterClass Atom
info: |
  The production Atom :: CharacterClass evaluates as follows:

  1. Evaluate CharacterClass to obtain a CharSet A and a Boolean invert.
  2. Call CharacterSetMatcher(A, invert) and return its Matcher result. 

  Runtime Semantics: CharacterSetMatcher Abstract Operation

  1. Return an internal Matcher closure that takes two arguments, a State x and
     a Continuation c, and performs the following steps when evaluated:
     [...]
     d. Let cc be Canonicalize(ch).
     [...]
---*/

assert(/^[\ud800\udc00]$/u.test('\ud800\udc00'));
assert.sameValue(
  /[\ud800\udc00]/u.test('\ud800'),
  false,
  '\\ud800 does not qualify as a class member'
);
assert.sameValue(
  /[\ud800\udc00]/u.test('\udc00'),
  false,
  '\\udc00 does not qualify as a class member'
);
