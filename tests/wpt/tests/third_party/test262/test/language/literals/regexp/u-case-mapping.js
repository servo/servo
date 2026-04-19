// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Case mapping of astral symbols
es6id: 21.2.2.8.2
info: |
    21.2.2.8.2 Runtime Semantics: Canonicalize ( ch )

    The abstract operation Canonicalize takes a character parameter ch and
    performs the following steps:

        1. If IgnoreCase is false, return ch.
        2. If Unicode is true,
           a. If the file CaseFolding.txt of the Unicode Character Database
              provides a simple or common case folding mapping for ch, return
              the result of applying that mapping to ch.
           b. Else, return ch.
---*/

assert.sameValue(
  /\u212a/i.test('k'),
  false,
  'Case mapping is not applied in the absence of the `u` flag'
);
assert.sameValue(
  /\u212a/i.test('K'),
  false,
  'Case mapping is not applied in the absence of the `u` flag'
);
assert.sameValue(
  /\u212a/u.test('k'),
  false,
  'Case mapping is not applied in the absence of the `i` flag'
);
assert.sameValue(
  /\u212a/u.test('K'),
  false,
  'Case mapping is not applied in the absence of the `i` flag'
);

assert(
  /\u212a/iu.test('k'),
  'Case mapping is applied in the presence of the `i` and `u` flags'
);
assert(
  /\u212a/iu.test('K'),
  'Case mapping is applied in the presence of the `i` and `u` flags'
);
