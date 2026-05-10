// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.towellformed
description: >
  The method should return a well-formed string.
info: |
  String.prototype.toWellFormed ( )

  1. Let O be ? RequireObjectCoercible(this value).
  2. Let S be ? ToString(O).
  3. Let strLen be the length of S.
  4. Let k be 0.
  5. Let result be the empty String.
  6. Repeat, while k < strLen,
    a. Let cp be CodePointAt(S, k).
    b. If cp.[[IsUnpairedSurrogate]] is true, then
      i. Set result to the string-concatenation of result and 0xFFFD (REPLACEMENT CHARACTER).
    c. Else,
      i. Set result to the string-concatenation of result and UTF16EncodeCodePoint(cp.[[CodePoint]]).
    d. Set k to k + cp.[[CodeUnitCount]].
  7. Return result.

features: [String.prototype.toWellFormed]
---*/
assert.sameValue(typeof String.prototype.toWellFormed, 'function');

var replacementChar = '\uFFFD';
var leadingPoo = '\uD83D';
var trailingPoo = '\uDCA9';
var wholePoo = leadingPoo + trailingPoo;

assert.sameValue(
  ('a' + leadingPoo + 'c' + leadingPoo + 'e').toWellFormed(),
  'a' + replacementChar + 'c' + replacementChar + 'e',
  'lone leading surrogates are replaced with the expected replacement character'
);
assert.sameValue(
  ('a' + trailingPoo + 'c' + trailingPoo + 'e').toWellFormed(),
  'a' + replacementChar + 'c' + replacementChar + 'e',
  'lone trailing surrogates are replaced with the expected replacement character'
);
assert.sameValue(
  ('a' + trailingPoo + leadingPoo + 'd').toWellFormed(),
  'a' + replacementChar + replacementChar + 'd',
  'a wrong-ordered surrogate pair is replaced with two replacement characters'
)

assert.sameValue('a💩c'.toWellFormed(), 'a💩c', 'a surrogate pair using a literal code point is already well-formed');
assert.sameValue('a\uD83D\uDCA9c'.toWellFormed(), 'a\uD83D\uDCA9c', 'a surrogate pair formed by escape sequences is already well-formed');
assert.sameValue(('a' + leadingPoo + trailingPoo + 'd').toWellFormed(), 'a' + wholePoo + 'd', 'a surrogate pair formed by concatenation is already well-formed');
assert.sameValue(wholePoo.slice(0, 1).toWellFormed(), replacementChar, 'a surrogate pair sliced to the leading surrogate is replaced with the expected replacement character');
assert.sameValue(wholePoo.slice(1).toWellFormed(), replacementChar, 'a surrogate pair sliced to the trailing surrogate is replaced with the expected replacement character');
assert.sameValue('abc'.toWellFormed(), 'abc', 'a latin-1 string is already well-formed');
assert.sameValue('a\u25A8c'.toWellFormed(), 'a\u25A8c', 'a string with a non-ASCII character is already well-formed');
