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
    ...
    DecimalDigits DecimalDigit
    ...

features: [numeric-separator-literal]
---*/

assert.sameValue(Number("1_0123456789"), NaN, "1_0123456789");
