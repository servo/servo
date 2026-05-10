// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.tofixed
description: Number.prototype.toFixed does not use ToString's cleaner rounding
info: |
  Number.prototype.toFixed ( fractionDigits )

  ...
  8. Else x < 10^21,
    a. Let n be an integer for which the exact mathematical value of n ÷ 10f - x is as close to zero as possible. If there are two such n, pick the larger n.
    b. If n = 0, let m be the String "0". Otherwise, let m be the String consisting of the digits of the decimal representation of n (in order, with no leading zeroes).
  ...
---*/

// Test from a note in the specification
assert.sameValue((1000000000000000128).toString(), "1000000000000000100");
assert.sameValue((1000000000000000128).toFixed(0), "1000000000000000128");
