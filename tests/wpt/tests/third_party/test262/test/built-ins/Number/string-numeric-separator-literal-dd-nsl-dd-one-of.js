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


  DecimalDigits ::
    DecimalDigit
    ...

  DecimalDigit :: one of
    0 1 2 3 4 5 6 7 8 9

features: [numeric-separator-literal]
---*/

assert.sameValue(Number("1_0"), NaN, "1_0");
assert.sameValue(Number("1_1"), NaN, "1_1");
assert.sameValue(Number("1_2"), NaN, "1_2");
assert.sameValue(Number("1_3"), NaN, "1_3");
assert.sameValue(Number("1_4"), NaN, "1_4");
assert.sameValue(Number("1_5"), NaN, "1_5");
assert.sameValue(Number("1_6"), NaN, "1_6");
assert.sameValue(Number("1_7"), NaN, "1_7");
assert.sameValue(Number("1_8"), NaN, "1_8");
assert.sameValue(Number("1_9"), NaN, "1_9");
