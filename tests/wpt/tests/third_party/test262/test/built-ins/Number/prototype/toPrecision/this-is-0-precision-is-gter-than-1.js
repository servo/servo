// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toprecision
description: >
  Return string value for this value = 0 and precision is > 1
info: |
  Number.prototype.toPrecision ( precision )

  1. Let x be ? thisNumberValue(this value).
  [...]
  5. Let s be the empty String.
  [...]
  9. If x = 0, then
    a. Let m be the String consisting of p occurrences of the code unit 0x0030
    (DIGIT ZERO).
    b. Let e be 0.
  [...]
  11. If e = p-1, return the concatenation of the Strings s and m.
  12. If e ≥ 0, then
    a. Let m be the concatenation of the first e+1 elements of m, the code unit
    0x002E (FULL STOP), and the remaining p- (e+1) elements of m.
  [...]
  14. Return the String that is the concatenation of s and m.
---*/

assert.sameValue(
  (0).toPrecision(2),
  "0.0",
  "(0).toPrecision(2)"
);

assert.sameValue(
  (0).toPrecision(7),
  "0.000000",
  "(0).toPrecision(7)"
);

assert.sameValue(
  (0).toPrecision(21),
  "0.00000000000000000000",
  "(0).toPrecision(21)"
);

assert.sameValue(
  (-0).toPrecision(2),
  "0.0",
  "(-0).toPrecision(2)"
);

assert.sameValue(
  (-0).toPrecision(7),
  "0.000000",
  "(-0).toPrecision(7)"
);

assert.sameValue(
  (-0).toPrecision(21),
  "0.00000000000000000000",
  "(-0).toPrecision(21)"
);
