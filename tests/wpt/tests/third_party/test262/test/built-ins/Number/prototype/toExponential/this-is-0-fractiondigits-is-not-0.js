// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toexponential
description: >
  Return string value for this value = 0 and fractionDigits != 0
info: |
  Number.prototype.toExponential ( fractionDigits )

  1. Let x be ? thisNumberValue(this value).
  [...]
  9. If x = 0, then
    a. Let m be the String consisting of f+1 occurrences of the code unit 0x0030
    (DIGIT ZERO).
    b. Let e be 0.
  [...]
  11. If f ≠ 0, then
    a. Let a be the first element of m, and let b be the remaining f elements of m.
    b. Let m be the concatenation of the three Strings a, ".", and b.
  12. If e = 0, then
    a. Let c be "+".
    b. Let d be "0".
  [...]
  14. Let m be the concatenation of the four Strings m, "e", c, and d.
  15. Return the concatenation of the Strings s and m. 
---*/

assert.sameValue((0).toExponential(1), "0.0e+0", "0 and 1");
assert.sameValue((0).toExponential(2), "0.00e+0", "0 and 2");
assert.sameValue((0).toExponential(7), "0.0000000e+0", "0 and 7");
assert.sameValue((0).toExponential(20), "0.00000000000000000000e+0", "0 and 20");

assert.sameValue((-0).toExponential(1), "0.0e+0", "-0 and 1");
assert.sameValue((-0).toExponential(2), "0.00e+0", "-0 and 2");
assert.sameValue((-0).toExponential(7), "0.0000000e+0", "-0 and 7");
assert.sameValue((-0).toExponential(20), "0.00000000000000000000e+0", "-0 and 20");

assert.sameValue((0.0).toExponential(4), "0.0000e+0", "0.0 and 4");
assert.sameValue((-0.0).toExponential(4), "0.0000e+0", "-0.0 and 4");
