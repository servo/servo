// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: NumericLiteralSeparator is not valid on string conversions for ToNumber operations
info: |
  DecimalDigits `.` DecimalDigits ExponentPart_opt `+` DecimalDigits

  NumericLiteralSeparator ::
    _

  DecimalLiteral ::
    DecimalIntegerLiteral . DecimalDigits_opt ExponentPart_opt

  DecimalDigits ::
    ...
    DecimalDigits NumericLiteralSeparator DecimalDigit

  SignedInteger ::
    ...
    + DecimalDigits
    ...

features: [numeric-separator-literal]
---*/

assert.sameValue(Number("1.0e+1_0"), NaN, "1.0e+1_0");
