// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.3
description: >
  Return single code unit value of the element at index position.
info: |
  21.1.3.3 String.prototype.codePointAt ( pos )

  1. Let O be RequireObjectCoercible(this value).
  2. Let S be ToString(O).
  3. ReturnIfAbrupt(S).
  4. Let position be ToInteger(pos).
  5. ReturnIfAbrupt(position).
  6. Let size be the number of elements in S.
  7. If position < 0 or position ≥ size, return undefined.
  8. Let first be the code unit value of the element at index position in the
  String S.
  9. If first < 0xD800 or first > 0xDBFF or position+1 = size, return first.
---*/

assert.sameValue('abc'.codePointAt(0), 97);
assert.sameValue('abc'.codePointAt(1), 98);
assert.sameValue('abc'.codePointAt(2), 99);

assert.sameValue('\uAAAA\uBBBB'.codePointAt(0), 0xAAAA);
assert.sameValue('\uD7FF\uAAAA'.codePointAt(0), 0xD7FF);
assert.sameValue('\uDC00\uAAAA'.codePointAt(0), 0xDC00);
assert.sameValue('\uAAAA\uBBBB'.codePointAt(0), 0xAAAA);

assert.sameValue('123\uD800'.codePointAt(3), 0xD800);
assert.sameValue('123\uDAAA'.codePointAt(3), 0xDAAA);
assert.sameValue('123\uDBFF'.codePointAt(3), 0xDBFF);
