// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.3
description: >
  Returns code point of LeadSurrogate if not followed by a valid TrailSurrogate.
info: |
  21.1.3.3 String.prototype.codePointAt ( pos )

  ...
  8. Let first be the code unit value of the element at index position in the
  String S.
  9. If first < 0xD800 or first > 0xDBFF or position+1 = size, return first.
  10. Let second be the code unit value of the element at index position+1 in
  the String S.
  11. If second < 0xDC00 or second > 0xDFFF, return first.
---*/

assert.sameValue('\uD800\uDBFF'.codePointAt(0), 0xD800);
assert.sameValue('\uD800\uE000'.codePointAt(0), 0xD800);

assert.sameValue('\uDAAA\uDBFF'.codePointAt(0), 0xDAAA);
assert.sameValue('\uDAAA\uE000'.codePointAt(0), 0xDAAA);

assert.sameValue('\uDBFF\uDBFF'.codePointAt(0), 0xDBFF);
assert.sameValue('\uDBFF\uE000'.codePointAt(0), 0xDBFF);

assert.sameValue('\uD800\u0000'.codePointAt(0), 0xD800);
assert.sameValue('\uD800\uFFFF'.codePointAt(0), 0xD800);

assert.sameValue('\uDAAA\u0000'.codePointAt(0), 0xDAAA);
assert.sameValue('\uDAAA\uFFFF'.codePointAt(0), 0xDAAA);

assert.sameValue('\uDBFF\uDBFF'.codePointAt(0), 0xDBFF);
assert.sameValue('\uDBFF\uFFFF'.codePointAt(0), 0xDBFF);
