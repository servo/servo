// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: ToString is applied BigInt values in an additive expression with another string
esid: prod-AdditiveExpression
info: |
  AdditiveExpression: AdditiveExpression + MultiplicativeExpression

  ...
  7. If Type(lprim) is String or Type(rprim) is String, then
    a. Let lstr be ? ToString(lprim).
    b. Let rstr be ? ToString(rprim).
    c. Return the String that is the result of concatenating lstr and rstr.
  ...

  ToString Applied to the BigInt Type

  1. If i is less than zero, return the String concatenation of the String "-" and ToString(-i).
  2. Return the String consisting of the code units of the digits of the decimal representation of i.
features: [BigInt]
---*/

assert.sameValue(-1n + "", "-1");
assert.sameValue("" + -1n, "-1");
assert.sameValue(0n + "", "0");
assert.sameValue("" + 0n, "0");
assert.sameValue(1n + "", "1");
assert.sameValue("" + 1n, "1");
assert.sameValue(123456789000000000000000n + "", "123456789000000000000000");
assert.sameValue("" + 123456789000000000000000n, "123456789000000000000000");
assert.sameValue(-123456789000000000000000n + "", "-123456789000000000000000");
assert.sameValue("" + -123456789000000000000000n, "-123456789000000000000000");
