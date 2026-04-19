// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 20.1.1.1
description: Invalid binary literals yield NaN
info: |
    BinaryIntegerLiteral ::
      0b BinaryDigits
      0B BinaryDigits
    BinaryDigits ::
      BinaryDigit
      BinaryDigits BinaryDigit
    BinaryDigit :: one of
      0 1
---*/

assert.sameValue(Number('0b2'), NaN, 'invalid digit');
assert.sameValue(Number('00b0'), NaN, 'leading zero');
assert.sameValue(Number('0b'), NaN, 'omitted digits');
assert.sameValue(Number('+0b1'), NaN, 'plus sign');
assert.sameValue(Number('-0b1'), NaN, 'minus sign');
assert.sameValue(Number('0b1.01'), NaN, 'fractional part');
assert.sameValue(Number('0b1e10'), NaN, 'exponent part');
assert.sameValue(Number('0b1e-10'), NaN, 'exponent part with a minus sign');
assert.sameValue(Number('0b1e+10'), NaN, 'exponent part with a plus sign');
