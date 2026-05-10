// Copyright 2016 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-applying-the-exp-operator
description: If base is −0 and exponent > 0 and exponent is an odd integer, the result is −0.
features: [exponentiation]
---*/


var base = -0;
var exponents = [];
exponents[0] = 1;
exponents[1] = 111;
exponents[2] = 111111;

for (var i = 0; i < exponents.length; i++) {
  assert.sameValue(base ** exponents[i], -0, base + " **  " + exponents[i]);
}
