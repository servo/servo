// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: NonZeroDigit NumericLiteralSeparator DecimalDigit
info: |
  NumericLiteralSeparator ::
    _

  DecimalIntegerLiteral ::
    ...
    NonZeroDigit NumericLiteralSeparator_opt DecimalDigits

  NonZeroDigit :: one of
    1 2 3 4 5 6 7 8 9

  DecimalDigits ::
    DecimalDigit
    ...

  DecimalDigit :: one of
    0 1 2 3 4 5 6 7 8 9

features: [numeric-separator-literal]
---*/

assert.sameValue(1_0, 10);
assert.sameValue(1_1, 11);
assert.sameValue(2_2, 22);
assert.sameValue(3_3, 33);
assert.sameValue(4_4, 44);
assert.sameValue(5_5, 55);
assert.sameValue(6_6, 66);
assert.sameValue(7_7, 77);
assert.sameValue(8_8, 88);
assert.sameValue(9_9, 99);


