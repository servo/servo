// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base is +0 and exponent > 0, the result is +0.
esid: sec-applying-the-exp-operator
---*/


var base = +0;
var exponent = new Array();
exponent[3] = Infinity;
exponent[2] = 1.7976931348623157E308; //largest finite number
exponent[1] = 1;
exponent[0] = 0.000000000000001;
var exponentnum = 4;

for (var i = 0; i < exponentnum; i++) {
  assert.sameValue(Math.pow(base, exponent[i]), 0, exponent[i]);
}
