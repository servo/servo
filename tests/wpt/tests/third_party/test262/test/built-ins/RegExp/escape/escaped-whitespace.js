// Copyright (C) 2024 Leo Balter, Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Escaped WhiteSpace characters (simple assertions)
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

  WhiteSpace ::
    <TAB> U+0009 CHARACTER TABULATION
    <VT> U+000B LINE TABULATION
    <FF> U+000C FORM FEED (FF)
    <ZWNBSP> U+FEFF ZERO WIDTH NO-BREAK SPACE
    <USP>

    U+0020 (SPACE) and U+00A0 (NO-BREAK SPACE) code points are part of <USP>
    Other USP U+202F NARROW NO-BREAK SPACE

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

const WhiteSpace = '\uFEFF\u0020\u00A0\u202F';

assert.sameValue(RegExp.escape('\uFEFF'), '\\ufeff', `whitespace \\uFEFF is escaped correctly to \\uFEFF`);
assert.sameValue(RegExp.escape('\u0020'), '\\x20', `whitespace \\u0020 is escaped correctly to \\x20`);
assert.sameValue(RegExp.escape('\u00A0'), '\\xa0', `whitespace \\u00A0 is escaped correctly to \\xA0`);
assert.sameValue(RegExp.escape('\u202F'), '\\u202f', `whitespace \\u202F is escaped correctly to \\u202F`);

assert.sameValue(RegExp.escape(WhiteSpace), '\\ufeff\\x20\\xa0\\u202f', `whitespaces are escaped correctly`);
