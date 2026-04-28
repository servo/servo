// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.iswellformed
description: >
  The method should return a boolean.
info: |
  String.prototype.isWellFormed ( )

  1. Let O be ? RequireObjectCoercible(this value).
  2. Let S be ? ToString(O).
  3. Return IsStringWellFormedUnicode(S).

features: [String.prototype.isWellFormed]
---*/
assert.sameValue(typeof String.prototype.isWellFormed, 'function');

var leadingPoo = '\uD83D';
var trailingPoo = '\uDCA9';
var wholePoo = leadingPoo + trailingPoo;

assert.sameValue(
  ('a' + leadingPoo + 'c' + leadingPoo + 'e').isWellFormed(),
  false,
  'lone leading surrogates are not well-formed'
);
assert.sameValue(
  ('a' + trailingPoo + 'c' + trailingPoo + 'e').isWellFormed(),
  false,
  'lone trailing surrogates are not well-formed'
);
assert.sameValue(
  ('a' + trailingPoo + leadingPoo + 'd').isWellFormed(),
  false,
  'a wrong-ordered surrogate pair is not well-formed'
)

assert.sameValue('a💩c'.isWellFormed(), true, 'a surrogate pair using a literal code point is well-formed');
assert.sameValue('a\uD83D\uDCA9c'.isWellFormed(), true, 'a surrogate pair formed by escape sequences is well-formed');
assert.sameValue(('a' + leadingPoo + trailingPoo + 'd').isWellFormed(), true, 'a surrogate pair formed by concatenation is well-formed');
assert.sameValue(wholePoo.slice(0, 1).isWellFormed(), false, 'a surrogate pair sliced to the leading surrogate is not well-formed');
assert.sameValue(wholePoo.slice(1).isWellFormed(), false, 'a surrogate pair sliced to the trailing surrogate is not well-formed');
assert.sameValue('abc'.isWellFormed(), true, 'a latin-1 string is well-formed');
assert.sameValue('a\u25A8c'.isWellFormed(), true, 'a non-ASCII character is well-formed');
