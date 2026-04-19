// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base is NaN and exponent is nonzero, the result is NaN.
esid: sec-applying-the-exp-operator
---*/


var base = NaN;
var exponent = new Array();
exponent[0] = -Infinity;
exponent[1] = -1.7976931348623157E308; //largest (by module) finite number
exponent[2] = -0.000000000000001;
exponent[3] = 0.000000000000001;
exponent[4] = 1.7976931348623157E308; //largest finite number
exponent[5] = +Infinity;
exponent[6] = NaN;
var exponentnum = 7;

for (var i = 0; i < exponentnum; i++) {
  assert.sameValue(
    Math.pow(base, exponent[i]),
    NaN,
    "(" + base + ", " + exponent[i] + ")"
  );
}
