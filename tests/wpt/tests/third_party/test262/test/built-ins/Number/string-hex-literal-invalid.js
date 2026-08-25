// Copyright (C) 2017 Ivan Vyshnevskyi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number-constructor-number-value
description: Invalid hex literals yield NaN
info: |
    HexIntegerLiteral ::
      0x HexDigits
      0X HexDigits
    HexDigits ::
      HexDigit
      HexDigits HexDigit
    HexDigit :: one of
      0 1 2 3 4 5 6 7 8 9 a b c d e f A B C D E F
---*/

assert.sameValue(Number('0xG'), NaN, 'invalid digit');
assert.sameValue(Number('00x0'), NaN, 'leading zero');
assert.sameValue(Number('0x'), NaN, 'omitted digits');
assert.sameValue(Number('+0x10'), NaN, 'plus sign');
assert.sameValue(Number('-0x10'), NaN, 'minus sign');
assert.sameValue(Number('0x10.01'), NaN, 'fractional part');
assert.sameValue(Number('0x1e-10'), NaN, 'exponent part with a minus sign');
assert.sameValue(Number('0x1e+10'), NaN, 'exponent part with a plus sign');
