// Copyright 2016 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-applying-the-exp-operator
description: If base is +0 and exponent > 0, the result is +0.
features: [exponentiation]
---*/


var base = +0;
var exponents = [];
exponents[3] = Infinity;
exponents[2] = 1.7976931348623157E308; //largest finite number
exponents[1] = 1;
exponents[0] = 0.000000000000001;

for (var i = 0; i < exponents.length; i++) {
  if ((base ** exponents[i]) !== +0) {
    throw new Test262Error("(" + base + " **  " + exponents[i] + ") !== +0");
  }
}
