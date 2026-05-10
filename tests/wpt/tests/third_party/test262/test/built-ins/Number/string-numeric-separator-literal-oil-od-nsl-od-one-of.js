// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: NumericLiteralSeparator is not valid on string conversions for ToNumber operations
info: |
  `0o` | `0O` OctalDigit NumericLiteralSeparator OctalDigit

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

features: [numeric-separator-literal]
---*/

assert.sameValue(Number("0o0_0"), NaN, "0o0_0");
assert.sameValue(Number("0o1_1"), NaN, "0o1_1");
assert.sameValue(Number("0o2_2"), NaN, "0o2_2");
assert.sameValue(Number("0o3_3"), NaN, "0o3_3");
assert.sameValue(Number("0o4_4"), NaN, "0o4_4");
assert.sameValue(Number("0o5_5"), NaN, "0o5_5");
assert.sameValue(Number("0o6_6"), NaN, "0o6_6");
assert.sameValue(Number("0o7_7"), NaN, "0o7_7");
