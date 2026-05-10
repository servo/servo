// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toexponential
description: >
  Return "0" if this value is 0 and ToInteger(fractionDigits) is 0
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
    [...]
  12. If e = 0, then
    a. Let c be "+".
    b. Let d be "0".
  [...]
  14. Let m be the concatenation of the four Strings m, "e", c, and d.
  15. Return the concatenation of the Strings s and m. 
---*/

assert.sameValue(Number.prototype.toExponential(0), "0e+0", "Number.prototype");

assert.sameValue((0).toExponential(0), "0e+0", "(0).toExponential(0)");
assert.sameValue((-0).toExponential(0), "0e+0", "(-0).toExponential(0)");

assert.sameValue((0).toExponential(-0), "0e+0", "(0).toExponential(-0)");
assert.sameValue((-0).toExponential(-0), "0e+0", "(-0).toExponential(-0)");
