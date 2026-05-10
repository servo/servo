// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-tonumber-applied-to-the-string-type
description: >
  The NSL does not affect strings parsed by parseFloat - StrDecimalDigits
info: |
  StrDecimalDigits :::
    DecimalDigit
    StrDecimalDigits DecimalDigit

features: [numeric-separator-literal]
---*/

assert.sameValue(parseFloat("123456789_0"), 123456789);
assert.sameValue(parseFloat("123456789_1"), 123456789);
assert.sameValue(parseFloat("123456789_2"), 123456789);
assert.sameValue(parseFloat("123456789_3"), 123456789);
assert.sameValue(parseFloat("123456789_4"), 123456789);
assert.sameValue(parseFloat("123456789_5"), 123456789);
assert.sameValue(parseFloat("123456789_6"), 123456789);
assert.sameValue(parseFloat("123456789_7"), 123456789);
assert.sameValue(parseFloat("123456789_8"), 123456789);
assert.sameValue(parseFloat("123456789_9"), 123456789);
