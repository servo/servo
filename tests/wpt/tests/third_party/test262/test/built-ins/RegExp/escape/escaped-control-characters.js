// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-encodeforregexescape
description: Encodes control characters with their ControlEscape sequences
info: |
  EncodeForRegExpEscape ( c )

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

const controlCharacters = '\t\n\v\f\r';
const expectedEscapedCharacters = '\\t\\n\\v\\f\\r';

assert.sameValue(RegExp.escape(controlCharacters), expectedEscapedCharacters, 'Control characters are correctly escaped');
