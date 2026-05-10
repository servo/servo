// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: >
  `0x` | `0X` HexDigits NumericLiteralSeparator HexDigit
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

  HexIntegerLiteral ::
    0x HexDigits
    0X HexDigits

  HexDigits ::
    HexDigit
    HexDigits HexDigit
    HexDigits NumericLiteralSeparator HexDigit

  HexDigit::one of
    0 1 2 3 4 5 6 7 8 9 a b c d e f A B C D E F

features: [BigInt, numeric-separator-literal]
---*/

assert.sameValue(0x01_0n, 0x010n);
assert.sameValue(0X01_0n, 0X010n);
