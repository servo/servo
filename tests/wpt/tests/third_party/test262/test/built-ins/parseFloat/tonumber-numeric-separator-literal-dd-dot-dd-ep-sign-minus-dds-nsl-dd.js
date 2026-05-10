// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-tonumber-applied-to-the-string-type
description: >
  The NSL does not affect strings parsed by parseFloat - StrDecimalDigits . StrDecimalDigits StrExponentPart
info: |
  StrUnsignedDecimalLiteral :::
    StrDecimalDigits . StrDecimalDigits StrExponentPart

  StrDecimalDigits :::
    DecimalDigit
    StrDecimalDigits NumericLiteralSeparator DecimalDigit


features: [numeric-separator-literal]
---*/

assert.sameValue(parseFloat("1.0e-10_0"), 1.0e-10);
