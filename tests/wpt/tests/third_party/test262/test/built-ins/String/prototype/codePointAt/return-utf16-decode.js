// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.3
description: >
  Return UTF16 Decode value of the lead and trail elements at index position.
info: |
  21.1.3.3 String.prototype.codePointAt ( pos )

  ...
  8. Let first be the code unit value of the element at index position in the
  String S.
  9. If first < 0xD800 or first > 0xDBFF or position+1 = size, return first.
  10. Let second be the code unit value of the element at index position+1 in
  the String S.
  11. If second < 0xDC00 or second > 0xDFFF, return first.
  12. Return UTF16Decode(first, second).

  10.1.2 Static Semantics: UTF16Decode( lead, trail )

  Two code units, lead and trail, that form a UTF-16 surrogate pair are
  converted to a code point by performing the following steps:

  1. Assert: 0xD800 ≤ lead ≤ 0xDBFF and 0xDC00 ≤ trail ≤ 0xDFFF.
  2. Let cp be (lead – 0xD800) × 1024 + (trail – 0xDC00) + 0x10000.
  3. Return the code point cp.
---*/

assert.sameValue('\uD800\uDC00'.codePointAt(0), 65536, 'U+10000');
assert.sameValue('\uD800\uDDD0'.codePointAt(0), 66000, 'U+101D0');
assert.sameValue('\uD800\uDFFF'.codePointAt(0), 66559, 'U+103FF');

assert.sameValue('\uDAAA\uDC00'.codePointAt(0), 763904, 'U+BA800');
assert.sameValue('\uDAAA\uDDD0'.codePointAt(0), 764368, 'U+BA9D0');
assert.sameValue('\uDAAA\uDFFF'.codePointAt(0), 764927, 'U+BABFF');

assert.sameValue('\uDBFF\uDC00'.codePointAt(0), 1113088, 'U+10FC00');
assert.sameValue('\uDBFF\uDDD0'.codePointAt(0), 1113552, 'U+10FDD0');
assert.sameValue('\uDBFF\uDFFF'.codePointAt(0), 1114111, 'U+10FFFF');
