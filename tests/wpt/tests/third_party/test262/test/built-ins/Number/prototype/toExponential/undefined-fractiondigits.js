// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toexponential
description: >
  Handle undefined fractionDigits, not only casting it to 0
info: |
  Number.prototype.toExponential ( fractionDigits )

  1. Let x be ? thisNumberValue(this value).
  2. Let f be ? ToInteger(fractionDigits).
  [...]
  10. Else x ≠ 0,
    a. If fractionDigits is not undefined, then
      i. Let e and n be integers such that 10f ≤ n < 10f+1 and for which the
         exact mathematical value of n × 10e-f - x is as close to zero as
         possible. If there are two such sets of e and n, pick the e and n for
         which n × 10e-f is larger.
    b. Else fractionDigits is undefined,
      i. Let e, n, and f be integers such that f ≥ 0, 10f ≤ n < 10f+1, the
         Number value for n × 10e-f is x, and f is as small as possible. Note
         that the decimal representation of n has f+1 digits, n is not divisible
         by 10, and the least significant digit of n is not necessarily uniquely
         determined by these criteria.
---*/

assert.sameValue((123.456).toExponential(undefined), "1.23456e+2", "undefined");
assert.sameValue((123.456).toExponential(), "1.23456e+2", "no arg");
assert.sameValue((123.456).toExponential(0), "1e+2", "0");

assert.sameValue((1.1e-32).toExponential(undefined), "1.1e-32", "undefined");
assert.sameValue((1.1e-32).toExponential(), "1.1e-32", "no arg");
assert.sameValue((1.1e-32).toExponential(0), "1e-32", "0");

assert.sameValue((100).toExponential(undefined), "1e+2", "undefined");
assert.sameValue((100).toExponential(), "1e+2", "no arg");
assert.sameValue((100).toExponential(0), "1e+2", "0");
