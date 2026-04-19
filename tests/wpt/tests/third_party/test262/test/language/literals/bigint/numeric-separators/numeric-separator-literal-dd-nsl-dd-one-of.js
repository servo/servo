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


  DecimalDigits ::
    DecimalDigit
    ...

  DecimalDigit :: one of
    0 1 2 3 4 5 6 7 8 9

features: [BigInt, numeric-separator-literal]
---*/

assert.sameValue(1_0n, 10n);
assert.sameValue(1_1n, 11n);
assert.sameValue(1_2n, 12n);
assert.sameValue(1_3n, 13n);
assert.sameValue(1_4n, 14n);
assert.sameValue(1_5n, 15n);
assert.sameValue(1_6n, 16n);
assert.sameValue(1_7n, 17n);
assert.sameValue(1_8n, 18n);
assert.sameValue(1_9n, 19n);
