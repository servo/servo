// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: DecimalDigits NumericLiteralSeparator DecimalDigit
info: |
  NumericLiteralSeparator :: _

  DecimalDigits ::
    ...
    DecimalDigits NumericLiteralSeparator DecimalDigit
features: [numeric-separator-literal]
---*/

assert.sameValue(Number("123456789_0"), NaN, "123456789_0");
assert.sameValue(Number("123456789_1"), NaN, "123456789_1");
assert.sameValue(Number("123456789_2"), NaN, "123456789_2");
assert.sameValue(Number("123456789_3"), NaN, "123456789_3");
assert.sameValue(Number("123456789_4"), NaN, "123456789_4");
assert.sameValue(Number("123456789_5"), NaN, "123456789_5");
assert.sameValue(Number("123456789_6"), NaN, "123456789_6");
assert.sameValue(Number("123456789_7"), NaN, "123456789_7");
assert.sameValue(Number("123456789_8"), NaN, "123456789_8");
assert.sameValue(Number("123456789_9"), NaN, "123456789_9");
