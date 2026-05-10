// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base is −0 and exponent < 0 and exponent is an odd integer, the result is −∞.
esid: sec-applying-the-exp-operator
---*/


var base = -0;
var exponent = new Array();
exponent[2] = -1;
exponent[1] = -111;
exponent[0] = -111111;
var exponentnum = 3;

for (var i = 0; i < exponentnum; i++) {
  if (Math.pow(base, exponent[i]) !== -Infinity) {
    throw new Test262Error("#1: Math.pow(" + base + ", " + exponent[i] + ") !== -Infinity");
  }
}
