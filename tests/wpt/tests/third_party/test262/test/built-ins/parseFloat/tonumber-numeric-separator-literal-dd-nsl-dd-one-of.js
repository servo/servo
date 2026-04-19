// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-tonumber-applied-to-the-string-type
description: >
  The NSL does not affect strings parsed by parseFloat - StrUnsignedDecimalLiteral
info: |
  StrStrUnsignedDecimalLiteral :::
    StrUnsignedDecimalLiteral


  StrDecimalDigits :::
    DecimalDigit
    ...

  DecimalDigit ::: one of
    0 1 2 3 4 5 6 7 8 9

features: [numeric-separator-literal]
---*/

assert.sameValue(parseFloat("1_0"), 1);
assert.sameValue(parseFloat("1_1"), 1);
assert.sameValue(parseFloat("1_2"), 1);
assert.sameValue(parseFloat("1_3"), 1);
assert.sameValue(parseFloat("1_4"), 1);
assert.sameValue(parseFloat("1_5"), 1);
assert.sameValue(parseFloat("1_6"), 1);
assert.sameValue(parseFloat("1_7"), 1);
assert.sameValue(parseFloat("1_8"), 1);
assert.sameValue(parseFloat("1_9"), 1);
