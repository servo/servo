// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base is −0 and exponent < 0 and exponent is not an odd integer, the result is +∞.
esid: sec-applying-the-exp-operator
---*/


var base = -0;
var exponent = new Array();
exponent[4] = -0.000000000000001;
exponent[3] = -2;
exponent[2] = -Math.PI;
exponent[1] = -1.7976931348623157E308; //largest (by module) finite number
exponent[0] = -Infinity;
var exponentnum = 5;

for (var i = 0; i < exponentnum; i++) {
  if (Math.pow(base, exponent[i]) !== +Infinity) {
    throw new Test262Error("#1: Math.pow(" + base + ", " + exponent[i] + ") !== +Infinity");
  }
}
