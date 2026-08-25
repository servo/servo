// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base is −∞ and exponent < 0 and exponent is an odd integer, the result is −0.
esid: sec-applying-the-exp-operator
---*/


var base = -Infinity;
var exponent = new Array();
exponent[2] = -1;
exponent[1] = -111;
exponent[0] = -111111;
var exponentnum = 3;

for (var i = 0; i < exponentnum; i++) {
  assert.sameValue(Math.pow(base, exponent[i]), -0, exponent[i]);
}
