// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-NumericLiteralSeparator
description: NonZeroDigit NumericLiteralSeparator DecimalDigit
info: |
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

assert.sameValue(1_0, 10);
assert.sameValue(1_1, 11);
assert.sameValue(1_2, 12);
assert.sameValue(1_3, 13);
assert.sameValue(1_4, 14);
assert.sameValue(1_5, 15);
assert.sameValue(1_6, 16);
assert.sameValue(1_7, 17);
assert.sameValue(1_8, 18);
assert.sameValue(1_9, 19);
