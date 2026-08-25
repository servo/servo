// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Escaped other punctuators characters
info: |
  EncodeForRegExpEscape ( c )

  ...
  3. Let otherPunctuators be the string-concatenation of ",-=<>#&!%:;@~'`" and the code unit 0x0022 (QUOTATION MARK).
  4. Let toEscape be StringToCodePoints(otherPunctuators).
  5. If toEscape contains c, (...), then
    a. If c ≤ 0xFF, then
      i. Let hex be Number::toString(𝔽(c), 16).
      ii. Return the string-concatenation of the code unit 0x005C (REVERSE SOLIDUS), "x", and StringPad(hex, 2, "0", START).
    b. Let escaped be the empty String.
    c. Let codeUnits be UTF16EncodeCodePoint(c).
    d. For each code unit cu of codeUnits, do
      i. Set escaped to the string-concatenation of escaped and UnicodeEscape(cu).
    e. Return escaped.
  6. Return UTF16EncodeCodePoint(c).

  codePoints
  0x002c ,
  0x002d -
  0x003d =
  0x003c <
  0x003e >
  0x0023 #
  0x0026 &
  0x0021 !
  0x0025 %
  0x003a :
  0x003b ;
  0x0040 @
  0x007e ~
  0x0027 '
  0x0060 `
  0x0022 "
features: [RegExp.escape]
---*/

const otherPunctuators = ",-=<>#&!%:;@~'`\"";

// Return the string-concatenation of the code unit 0x005C (REVERSE SOLIDUS), "x", and StringPad(hex, 2, "0", START).
for (const c of otherPunctuators) {
  const expected = `\\x${c.codePointAt(0).toString(16)}`;
  assert.sameValue(RegExp.escape(c), expected, `${c} is escaped correctly`);
}

const otherPunctuatorsExpected = "\\x2c\\x2d\\x3d\\x3c\\x3e\\x23\\x26\\x21\\x25\\x3a\\x3b\\x40\\x7e\\x27\\x60\\x22";

assert.sameValue(
  RegExp.escape(otherPunctuators),
  otherPunctuatorsExpected,
  'all other punctuators are escaped correctly'
);
