// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: NonZeroDigit NumericLiteralSeparator DecimalDigit
info: |
  NumericLiteral ::
    DecimalIntegerLiteral BigIntLiteralSuffix
    NumericLiteralBase BigIntLiteralSuffix

  NumericLiteralBase ::
    BinaryIntegerLiteral
    OctalIntegerLiteral
    HexIntegerLiteral

  BigIntLiteralSuffix :: n

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

features: [BigInt, numeric-separator-literal]
---*/

assert.sameValue(1_0n, 10n);
assert.sameValue(1_1n, 11n);
assert.sameValue(2_2n, 22n);
assert.sameValue(3_3n, 33n);
assert.sameValue(4_4n, 44n);
assert.sameValue(5_5n, 55n);
assert.sameValue(6_6n, 66n);
assert.sameValue(7_7n, 77n);
assert.sameValue(8_8n, 88n);
assert.sameValue(9_9n, 99n);


