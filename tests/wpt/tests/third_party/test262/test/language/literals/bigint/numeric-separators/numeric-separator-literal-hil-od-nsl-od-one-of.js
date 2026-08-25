// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: >
  `0x` | `0X` HexDigit NumericLiteralSeparator HexDigit
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

assert.sameValue(0x0_0n, 0x00n);
assert.sameValue(0x1_1n, 0x11n);
assert.sameValue(0x2_2n, 0x22n);
assert.sameValue(0x3_3n, 0x33n);
assert.sameValue(0x4_4n, 0x44n);
assert.sameValue(0x5_5n, 0x55n);
assert.sameValue(0x6_6n, 0x66n);
assert.sameValue(0x7_7n, 0x77n);
assert.sameValue(0x8_8n, 0x88n);
assert.sameValue(0x9_9n, 0x99n);
assert.sameValue(0xa_a, 0xaa);
assert.sameValue(0xb_b, 0xbb);
assert.sameValue(0xc_c, 0xcc);
assert.sameValue(0xd_d, 0xdd);
assert.sameValue(0xe_e, 0xee);
assert.sameValue(0xf_f, 0xff);
assert.sameValue(0xA_A, 0xAA);
assert.sameValue(0xB_B, 0xBB);
assert.sameValue(0xC_C, 0xCC);
assert.sameValue(0xD_D, 0xDD);
assert.sameValue(0xE_E, 0xEE);
assert.sameValue(0xF_F, 0xFF);
