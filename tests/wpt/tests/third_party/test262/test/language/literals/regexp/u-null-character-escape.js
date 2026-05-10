// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-CharacterEscape
description: >
  Null character escape is permitted in Unicode patterns.
info: |
  CharacterEscape[U] ::
    ControlEscape
    c ControlLetter
    0 [lookahead ∉ DecimalDigit]
    HexEscapeSequence
    RegExpUnicodeEscapeSequence[?U]
    IdentityEscape[?U]

  DecimalDigit :: one of
    0 1 2 3 4 5 6 7 8 9
---*/

var nullChar = String.fromCharCode(0);
assert.sameValue(/\0/u.exec(nullChar)[0], nullChar);
assert(/^\0a$/u.test('\0a'));
assert.sameValue('\x00②'.match(/\0②/u)[0], '\x00②');
assert.sameValue('\u0000፬'.search(/\0፬$/u), 0);
