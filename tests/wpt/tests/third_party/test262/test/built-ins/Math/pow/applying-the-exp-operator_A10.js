// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If abs(base) < 1 and exponent is −∞, the result is +∞.
esid: sec-applying-the-exp-operator
---*/


var exponent = -Infinity;
var base = new Array();
base[0] = 0.999999999999999;
base[1] = 0.5;
base[2] = +0;
base[3] = -0;
base[4] = -0.5;
base[5] = -0.999999999999999;
var basenum = 6;

for (var i = 0; i < basenum; i++) {
  if (Math.pow(base[i], exponent) !== +Infinity) {
    throw new Test262Error("#1: Math.pow(" + base[i] + ", " + exponent + ") !== +Infinity");
  }
}
