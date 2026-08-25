// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: NumericLiteralSeparator is not valid on string conversions for ToNumber operations
info: |
  `0x` | `0X` HexDigit NumericLiteralSeparator HexDigit

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

assert.sameValue(Number("0x0_0"), NaN, "0x0_0");
assert.sameValue(Number("0x1_1"), NaN, "0x1_1");
assert.sameValue(Number("0x2_2"), NaN, "0x2_2");
assert.sameValue(Number("0x3_3"), NaN, "0x3_3");
assert.sameValue(Number("0x4_4"), NaN, "0x4_4");
assert.sameValue(Number("0x5_5"), NaN, "0x5_5");
assert.sameValue(Number("0x6_6"), NaN, "0x6_6");
assert.sameValue(Number("0x7_7"), NaN, "0x7_7");
assert.sameValue(Number("0x8_8"), NaN, "0x8_8");
assert.sameValue(Number("0x9_9"), NaN, "0x9_9");
assert.sameValue(Number("0xa_a"), NaN, "0xa_a");
assert.sameValue(Number("0xb_b"), NaN, "0xb_b");
assert.sameValue(Number("0xc_c"), NaN, "0xc_c");
assert.sameValue(Number("0xd_d"), NaN, "0xd_d");
assert.sameValue(Number("0xe_e"), NaN, "0xe_e");
assert.sameValue(Number("0xf_f"), NaN, "0xf_f");
assert.sameValue(Number("0xA_A"), NaN, "0xA_A");
assert.sameValue(Number("0xB_B"), NaN, "0xB_B");
assert.sameValue(Number("0xC_C"), NaN, "0xC_C");
assert.sameValue(Number("0xD_D"), NaN, "0xD_D");
assert.sameValue(Number("0xE_E"), NaN, "0xE_E");
assert.sameValue(Number("0xF_F"), NaN, "0xF_F");
