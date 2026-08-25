// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Escaped lineterminator characters (simple assertions)
info: |
  EncodeForRegExpEscape ( c )

  ...
  3. Let otherPunctuators be the string-concatenation of ",-=<>#&!%:;@~'`" and the code unit 0x0022 (QUOTATION MARK).
  4. Let toEscape be StringToCodePoints(otherPunctuators).
  5. If toEscape ..., c is matched by WhiteSpace or LineTerminator, ..., then
    a. If c ≤ 0xFF, then
      i. Let hex be Number::toString(𝔽(c), 16).
      ii. Return the string-concatenation of the code unit 0x005C (REVERSE SOLIDUS), "x", and StringPad(hex, 2, "0", START).
    b. Let escaped be the empty String.
    c. Let codeUnits be UTF16EncodeCodePoint(c).
    d. For each code unit cu of codeUnits, do
      i. Set escaped to the string-concatenation of escaped and UnicodeEscape(cu).
    e. Return escaped.
  6. Return UTF16EncodeCodePoint(c).

  LineTerminator ::
    <LF>
    <CR>
    <LS>
    <PS>

  Exceptions:

    2. If c is the code point listed in some cell of the “Code Point” column of Table 64, then
    a. Return the string-concatenation of 0x005C (REVERSE SOLIDUS) and the string in the “ControlEscape” column of the row whose “Code Point” column contains c.

  ControlEscape, Numeric Value, Code Point, Unicode Name, Symbol
  t 9 U+0009 CHARACTER TABULATION <HT>
  n 10 U+000A LINE FEED (LF) <LF>
  v 11 U+000B LINE TABULATION <VT>
  f 12 U+000C FORM FEED (FF) <FF>
  r 13 U+000D CARRIAGE RETURN (CR) <CR>
features: [RegExp.escape]
---*/

assert.sameValue(RegExp.escape('\u2028'), '\\u2028', 'line terminator \\u2028 is escaped correctly to \\u2028');
assert.sameValue(RegExp.escape('\u2029'), '\\u2029', 'line terminator \\u2029 is escaped correctly to \\u2029'); 

assert.sameValue(RegExp.escape('\u2028\u2029'), '\\u2028\\u2029', 'line terminators are escaped correctly');
assert.sameValue(RegExp.escape('\u2028a\u2029a'), '\\u2028a\\u2029a', 'mixed line terminators are escaped correctly');
