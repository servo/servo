// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base is +∞ and exponent > 0, the result is +∞.
esid: sec-applying-the-exp-operator
---*/


var base = +Infinity;
var exponent = new Array();
exponent[3] = Infinity;
exponent[2] = 1.7976931348623157E308; //largest (by module) finite number
exponent[1] = 1;
exponent[0] = 0.000000000000001;
var exponentnum = 4;

for (var i = 0; i < exponentnum; i++) {
  if (Math.pow(base, exponent[i]) !== +Infinity) {
    throw new Test262Error("#1: Math.pow(" + base + ", " + exponent[i] + ") !== +Infinity");
  }
}
