// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If abs(base) < 1 and exponent is +∞, the result is +0.
esid: sec-applying-the-exp-operator
---*/


var exponent = +Infinity;
var base = new Array();
base[0] = 0.999999999999999;
base[1] = 0.5;
base[2] = +0;
base[3] = -0;
base[4] = -0.5;
base[5] = -0.999999999999999;
var basenum = 6;

for (var i = 0; i < basenum; i++) {
  assert.sameValue(Math.pow(base[i], exponent), 0, exponent);
}
