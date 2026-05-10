// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If exponent is NaN, the result is NaN.
esid: sec-applying-the-exp-operator
---*/


var exponent = NaN;
var base = new Array();
base[0] = -Infinity;
base[1] = -1.7976931348623157E308; //largest (by module) finite number
base[2] = -0.000000000000001;
base[3] = -0;
base[4] = +0
base[5] = 0.000000000000001;
base[6] = 1.7976931348623157E308; //largest finite number
base[7] = +Infinity;
base[8] = NaN;
base[9] = 1;
var basenum = 10;

for (var i = 0; i < basenum; i++) {
  assert.sameValue(
    Math.pow(base[i], exponent),
    NaN,
    base[i]
  );
}
