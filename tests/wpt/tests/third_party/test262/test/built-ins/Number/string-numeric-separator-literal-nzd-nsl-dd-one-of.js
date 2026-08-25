// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: NumericLiteralSeparator is not valid on string conversions for ToNumber operations
info: |
  NonZeroDigit NumericLiteralSeparator DecimalDigits

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

assert.sameValue(Number("1_0"), NaN, "1_0");
assert.sameValue(Number("1_1"), NaN, "1_1");
assert.sameValue(Number("2_2"), NaN, "2_2");
assert.sameValue(Number("3_3"), NaN, "3_3");
assert.sameValue(Number("4_4"), NaN, "4_4");
assert.sameValue(Number("5_5"), NaN, "5_5");
assert.sameValue(Number("6_6"), NaN, "6_6");
assert.sameValue(Number("7_7"), NaN, "7_7");
assert.sameValue(Number("8_8"), NaN, "8_8");
assert.sameValue(Number("9_9"), NaN, "9_9");
