// Copyright (c) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.issafeinteger
description: >
  Return false if argument is not an integer
info: |
  Number.isSafeInteger ( number )

  [...]
  3. Let integer be ToInteger(number).
  4. If integer is not equal to number, return false.
  [...]
---*/

assert.sameValue(Number.isSafeInteger(1.1), false, "1.1");
assert.sameValue(Number.isSafeInteger(0.000001), false, "0.000001");
assert.sameValue(Number.isSafeInteger(-0.000001), false, "-0.000001");
assert.sameValue(Number.isSafeInteger(11e-1), false, "11e-1");
