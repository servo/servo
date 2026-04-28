// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-tonumber-applied-to-the-string-type
description: >
  The NSL does not affect strings parsed by parseFloat - DecimalDigit
info: |
  StrDecimalDigits :::
    DecimalDigit
    StrDecimalDigits DecimalDigit

features: [numeric-separator-literal]
---*/

assert.sameValue(parseFloat("1_0123456789"), 1);
