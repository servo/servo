// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If abs(base) > 1 and exponent is −∞, the result is +0.
esid: sec-applying-the-exp-operator
---*/


var exponent = -Infinity;
var base = new Array();
base[0] = -Infinity;
base[1] = -1.7976931348623157E308; //largest (by module) finite number
base[2] = -1.000000000000001;
base[3] = 1.000000000000001;
base[4] = 1.7976931348623157E308; //largest finite number
base[5] = +Infinity;
var basenum = 6;

for (var i = 0; i < basenum; i++) {
  assert.sameValue(Math.pow(base[i], exponent), 0, base[i]);
}
