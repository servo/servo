// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 11.8.6.1
description: Template values of UTF-16 escape sequences
info: |
    The TV of TemplateCharacter :: \ EscapeSequence is the SV of
    EscapeSequence.
    The SV of UnicodeEscapeSequence :: u{ HexDigits } is the UTF16Encoding
    (10.1.1) of the MV of HexDigits.
    The TRV of UnicodeEscapeSequence :: u Hex4Digits is the sequence consisting
    of code unit value 0x0075 followed by TRV of Hex4Digits.
    The TRV of UnicodeEscapeSequence :: u{ HexDigits } is the sequence
    consisting of code unit value 0x0075 followed by code unit value 0x007B
    followed by TRV of HexDigits followed by code unit value 0x007D.
---*/

var calls;

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], 'b', 'u Hex4Digits template value');
  assert.sameValue(s.raw[0], '\\u0062', 'u Hex4Digits template raw value');
})`\u0062`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], 'b', 'u{ HexDigits } template value');
  assert.sameValue(
    s.raw[0], '\\u{62}', 'u{ Hex4Digits } template raw value'
  );
})`\u{62}`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(
    s[0], 'b', 'u{ HexDigits } template value (with leading zeros)'
  );
  assert.sameValue(
    s.raw[0],
    '\\u{000062}',
    'u{ HexDigits } template raw value (with leading zeros)'
  );
})`\u{000062}`;
assert.sameValue(calls, 1);
