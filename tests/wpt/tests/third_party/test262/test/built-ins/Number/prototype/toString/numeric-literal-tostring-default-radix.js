// Copyright 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.tostring
description: >
  If radix is undefined the Number 10 is used as the value of radix.
info: |
  If radix is undefined, let radixNumber be 10.
  ...
  If radixNumber = 10, return ! ToString(x).
  Return the String representation of this Number value using the radix specified by radixNumber. Letters a-z are used for digits with values 10 through 35. The precise algorithm is implementation-defined, however the algorithm should be a generalization of that specified in 6.1.6.1.20.

  The optional radix should be an integer value in the inclusive range 2 to 36. If radix is undefined the Number 10 is used as the value of radix.
---*/

assert.sameValue(0..toString(), "0");
assert.sameValue(1..toString(), "1");
assert.sameValue(NaN.toString(), "NaN");
assert.sameValue(Infinity.toString(), "Infinity");
