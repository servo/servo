// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base is −∞ and exponent < 0 and exponent is not an odd integer, the result is +0.
esid: sec-applying-the-exp-operator
---*/


var base = -Infinity;
var exponent = new Array();
exponent[4] = -0.000000000000001;
exponent[3] = -2;
exponent[2] = -Math.PI;
exponent[1] = -1.7976931348623157E308; //largest (by module) finite number
exponent[0] = -Infinity;
var exponentnum = 5;

for (var i = 0; i < exponentnum; i++) {
  assert.sameValue(Math.pow(base, exponent[i]), 0, exponent[i]);
}
