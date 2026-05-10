// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: >
  `0o` | `0O` OctalDigit NumericLiteralSeparator OctalDigit
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

  OctalIntegerLiteral ::
    0o OctalDigits
    0O OctalDigits

  OctalDigits ::
    OctalDigit
    OctalDigits OctalDigit
    OctalDigits NumericLiteralSeparator OctalDigit

  OctalDigit :: one of
    0 1 2 3 4 5 6 7

features: [BigInt, numeric-separator-literal]
---*/

assert.sameValue(0o0_0n, 0o00n);
assert.sameValue(0o1_1n, 0o11n);
assert.sameValue(0o2_2n, 0o22n);
assert.sameValue(0o3_3n, 0o33n);
assert.sameValue(0o4_4n, 0o44n);
assert.sameValue(0o5_5n, 0o55n);
assert.sameValue(0o6_6n, 0o66n);
assert.sameValue(0o7_7n, 0o77n);
