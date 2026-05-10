// Copyright 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.tostring
description: >
  Calling toString(radix) (28)
info: |
  Return the String representation of this Number value using the radix specified by radixNumber. Letters a-z are used for digits with values 10 through 35. The precise algorithm is implementation-defined, however the algorithm should be a generalization of that specified in sec-numeric-types-number-tostring.
---*/

assert.sameValue(0..toString(28), "0");
assert.sameValue(1..toString(28), "1");
assert.sameValue(NaN.toString(28), "NaN");
assert.sameValue(Infinity.toString(28), "Infinity");
