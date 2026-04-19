// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base is +∞ and exponent < 0, the result is +0.
esid: sec-applying-the-exp-operator
---*/


var base = +Infinity;
var exponent = new Array();
exponent[0] = -Infinity;
exponent[1] = -1.7976931348623157E308; //largest (by module) finite number
exponent[2] = -1;
exponent[3] = -0.000000000000001;
var exponentnum = 4;

for (var i = 0; i < exponentnum; i++) {
  assert.sameValue(Math.pow(base, exponent[i]), 0, exponent[i]);
}
