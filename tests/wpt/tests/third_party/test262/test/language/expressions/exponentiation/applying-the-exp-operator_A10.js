// Copyright 2016 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-applying-the-exp-operator
description: If abs(base) < 1 and exponent is −∞, the result is +∞.
---*/


var exponent = -Infinity;
var bases = [];
bases[0] = 0.999999999999999;
bases[1] = 0.5;
bases[2] = +0;
bases[3] = -0;
bases[4] = -0.5;
bases[5] = -0.999999999999999;

for (var i = 0; i < bases.length; i++) {
  if (Math.pow(bases[i], exponent) !== +Infinity) {
    throw new Test262Error("(" + bases[i] + " ** " + exponent + ") !== +Infinity");
  }
}
