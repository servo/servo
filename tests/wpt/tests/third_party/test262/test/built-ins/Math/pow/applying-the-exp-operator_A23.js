// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If base < 0 and base is finite and exponent is finite and exponent is not an integer, the result is NaN.
esid: sec-applying-the-exp-operator
---*/


var exponent = new Array();
var base = new Array();
base[0] = -1.7976931348623157E308; //largest (by module) finite number
base[1] = -Math.PI;
base[2] = -1;
base[3] = -0.000000000000001;
var basenum = 4;

exponent[0] = -Math.PI;
exponent[1] = -Math.E;
exponent[2] = -1.000000000000001;
exponent[3] = -0.000000000000001;
exponent[4] = 0.000000000000001;
exponent[5] = 1.000000000000001;
exponent[6] = Math.E;
exponent[7] = Math.PI;

var exponentnum = 8;

for (var i = 0; i < basenum; i++) {
  for (var j = 0; j < exponentnum; j++) {
    assert.sameValue(
      Math.pow(base[i], exponent[j]),
      NaN,
      "(" + base[i] + ", " + exponent[j] + ")"
    );
  }
}
