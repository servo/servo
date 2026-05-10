// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: >
  `0x` | `0X` HexDigit NumericLiteralSeparator HexDigit
info: |
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

features: [numeric-separator-literal]
---*/

assert.sameValue(0x0_0, 0x00);
assert.sameValue(0x1_1, 0x11);
assert.sameValue(0x2_2, 0x22);
assert.sameValue(0x3_3, 0x33);
assert.sameValue(0x4_4, 0x44);
assert.sameValue(0x5_5, 0x55);
assert.sameValue(0x6_6, 0x66);
assert.sameValue(0x7_7, 0x77);
assert.sameValue(0x8_8, 0x88);
assert.sameValue(0x9_9, 0x99);
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
