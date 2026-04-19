// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.2
description: >
  Returns the String value whose elements are, in order, the code unit for the
  numbers in the arguments list.
info: |
  String.fromCodePoint ( ...codePoints )

  1. Let result be the empty String.
  2. For each element next of codePoints, do
    ...
  3. Assert: If codePoints is empty, then result is the empty String.
  4. Return result.
features: [String.fromCodePoint]
---*/

assert.sameValue(String.fromCodePoint(0), '\x00');
assert.sameValue(String.fromCodePoint(42), '*');
assert.sameValue(String.fromCodePoint(65, 90), 'AZ');
assert.sameValue(String.fromCodePoint(0x404), '\u0404');
assert.sameValue(String.fromCodePoint(0x2F804), '\uD87E\uDC04');
assert.sameValue(String.fromCodePoint(194564), '\uD87E\uDC04');
assert.sameValue(
  String.fromCodePoint(0x1D306, 0x61, 0x1D307),
  '\uD834\uDF06a\uD834\uDF07'
);
assert.sameValue(String.fromCodePoint(1114111), '\uDBFF\uDFFF');
